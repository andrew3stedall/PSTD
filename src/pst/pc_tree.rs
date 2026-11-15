//! Property Context reader for a heap-on-node stored in a data tree
//! (MS-PST 2.3.3).
//!
//! A message whose properties do not fit one data block stores its heap in an
//! XBLOCK data tree: every leaf block carries its own page map and a heap id
//! names its block in the upper 16 bits. The single-block heap probe never
//! sees these heaps, so they are read here by structure.
//!
//! The reader fails closed: a heap, BTH or record layout that disagrees with
//! the specification yields an error and no properties.
use std::collections::HashSet;

use crate::error::{PstdError, PstdResult};
use crate::pst::bth::{BthEntry, BthHeader, BthMap};
use crate::pst::primitives::BlockId;
use crate::pst::tc_reader::{heap_allocation, load_data_blocks, TcSource};

const HEAP_SIGNATURE: u8 = 0xec;
const HEAP_CLIENT_PROPERTY_CONTEXT: u8 = 0xbc;
const BTH_HEADER_TYPE: u8 = 0xb5;
const PC_KEY_SIZE: u8 = 2;
const PC_VALUE_SIZE: u8 = 6;
const MAX_BTH_LEVELS: u8 = 8;
const MAX_PROPERTIES: usize = 4096;
const PTYP_OBJECT: u16 = 0x000d;

/// One property record: `(property id, property type, value or HNID)`.
type PcRecord = (u16, u16, u32);

/// Read the Property Context whose heap is the data tree rooted at `data_bid`.
/// Returns the properties and the number of heap blocks read.
pub(crate) fn read_property_context_tree(
    source: TcSource<'_>,
    data_bid: BlockId,
) -> PstdResult<(BthMap, usize)> {
    let heap = load_data_blocks(source, data_bid)?;
    let first = heap
        .first()
        .ok_or_else(|| parse_error("property context data tree has no blocks"))?;
    if first.len() < 8 || first[2] != HEAP_SIGNATURE || first[3] != HEAP_CLIENT_PROPERTY_CONTEXT {
        return Err(parse_error(
            "property context data tree is not a property-context heap",
        ));
    }
    let header = heap_allocation(&heap, u32_at(first, 4)?)?;
    if header.len() < 8 || header[0] != BTH_HEADER_TYPE {
        return Err(parse_error("property context root is not a BTH"));
    }
    let (key_size, value_size, levels) = (header[1], header[2], header[3]);
    if key_size != PC_KEY_SIZE || value_size != PC_VALUE_SIZE || levels > MAX_BTH_LEVELS {
        return Err(parse_error(format!(
            "unsupported property context BTH: key={key_size}, value={value_size}, levels={levels}"
        )));
    }
    let root = u32_at(header, 4)?;
    let mut records = Vec::new();
    if root != 0 {
        let mut seen = HashSet::new();
        append_records(&heap, root, levels, &mut seen, &mut records)?;
    }

    let mut ids = HashSet::with_capacity(records.len());
    let mut entries = Vec::with_capacity(records.len());
    for (property_id, property_type, value) in records {
        if !ids.insert(property_id) {
            return Err(parse_error(format!(
                "property context repeats property 0x{property_id:04x}"
            )));
        }
        entries.push(property_entry(&heap, property_id, property_type, value));
    }
    let map = BthMap {
        header: BthHeader {
            key_size: 4,
            value_size,
            entry_count: entries.len() as u16,
            root_allocation: root,
        },
        entries,
    };
    Ok((map, heap.len()))
}

fn append_records(
    heap: &[Vec<u8>],
    hid: u32,
    level: u8,
    seen: &mut HashSet<u32>,
    records: &mut Vec<PcRecord>,
) -> PstdResult<()> {
    if !seen.insert(hid) {
        return Err(parse_error(format!(
            "property context BTH revisits heap id 0x{hid:08x}"
        )));
    }
    let allocation = heap_allocation(heap, hid)?;
    let record = usize::from(PC_KEY_SIZE)
        + if level == 0 {
            usize::from(PC_VALUE_SIZE)
        } else {
            4
        };
    if allocation.len() % record != 0 {
        return Err(parse_error(
            "property context BTH node has a partial record",
        ));
    }
    for chunk in allocation.chunks_exact(record) {
        let key = u16::from_le_bytes([chunk[0], chunk[1]]);
        if level > 0 {
            let child = u32::from_le_bytes([chunk[2], chunk[3], chunk[4], chunk[5]]);
            append_records(heap, child, level - 1, seen, records)?;
            continue;
        }
        if records.len() == MAX_PROPERTIES {
            return Err(parse_error(format!(
                "property context holds more than {MAX_PROPERTIES} properties"
            )));
        }
        let property_type = u16::from_le_bytes([chunk[2], chunk[3]]);
        let value = u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
        records.push((key, property_type, value));
    }
    Ok(())
}

/// Same contract as the single-block heap reader: fixed-size values stay
/// inline, heap values are resolved across every heap block, and subnode
/// references stay unresolved for the subnode resolver.
fn property_entry(heap: &[Vec<u8>], property_id: u16, property_type: u16, value: u32) -> BthEntry {
    let tag = (u32::from(property_id) << 16) | u32::from(property_type);
    let inline = matches!(property_type, 0x0002 | 0x0003 | 0x0004 | 0x000a | 0x000b);
    let raw = value.to_le_bytes().to_vec();
    let (value, unresolved_hnid) = if property_type == PTYP_OBJECT || inline {
        (raw, None)
    } else if value == 0 {
        (Vec::new(), None)
    } else if value & 0x1f == 0 {
        match heap_allocation(heap, value) {
            Ok(bytes) => (bytes.to_vec(), None),
            Err(_) => (raw, Some(value)),
        }
    } else {
        (raw, Some(value))
    };
    BthEntry {
        unresolved_hnid,
        key: tag.to_le_bytes().to_vec(),
        value,
    }
}

fn parse_error(message: impl Into<String>) -> PstdError {
    PstdError::pst_parse(None, message.into())
}

fn u32_at(bytes: &[u8], offset: usize) -> PstdResult<u32> {
    bytes
        .get(offset..offset + 4)
        .map(|field| u32::from_le_bytes([field[0], field[1], field[2], field[3]]))
        .ok_or_else(|| parse_error("property context heap field outside its block"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pst::bbt::{BbtEntry, BbtIndex};
    use crate::pst::limits::ParserLimits;
    use crate::pst::mapi::PR_SUBJECT;
    use crate::pst::primitives::ByteOffset;
    use crate::pst::property_context::PropertyContext;
    use crate::pst::reader::PstByteReader;

    const XBLOCK_BID: u64 = 0x22;
    const FIRST_BID: u64 = 0x24;
    const SECOND_BID: u64 = 0x28;
    const PR_MESSAGE_SIZE: u32 = 0x0e08_0003;

    /// A heap block: allocations followed by the page map.
    fn heap_block(header: &[u8], allocations: &[&[u8]]) -> Vec<u8> {
        let mut block = header.to_vec();
        let mut offsets = vec![block.len() as u16];
        for allocation in allocations {
            block.extend_from_slice(allocation);
            offsets.push(block.len() as u16);
        }
        let page_map = block.len() as u16;
        block[0..2].copy_from_slice(&page_map.to_le_bytes());
        block.extend_from_slice(&(allocations.len() as u16).to_le_bytes());
        block.extend_from_slice(&0u16.to_le_bytes());
        for offset in offsets {
            block.extend_from_slice(&offset.to_le_bytes());
        }
        block
    }

    fn hid(block: u32, index: u32) -> u32 {
        (block << 16) | (index << 5)
    }

    fn utf16(value: &str) -> Vec<u8> {
        value
            .encode_utf16()
            .flat_map(|unit| unit.to_le_bytes())
            .collect()
    }

    fn leaf(property_id: u16, property_type: u16, value: u32) -> Vec<u8> {
        let mut record = property_id.to_le_bytes().to_vec();
        record.extend_from_slice(&property_type.to_le_bytes());
        record.extend_from_slice(&value.to_le_bytes());
        record
    }

    /// Two heap blocks: the BTH lives in block 0, the subject string in block 1.
    fn two_block_heap(leaf_records: Vec<u8>) -> (Vec<u8>, Vec<u8>) {
        let mut bth_header = vec![BTH_HEADER_TYPE, PC_KEY_SIZE, PC_VALUE_SIZE, 0];
        bth_header.extend_from_slice(&hid(0, 2).to_le_bytes());
        let mut first_header = vec![0, 0, HEAP_SIGNATURE, HEAP_CLIENT_PROPERTY_CONTEXT];
        first_header.extend_from_slice(&hid(0, 1).to_le_bytes());
        first_header.extend_from_slice(&[0; 4]);
        let first = heap_block(&first_header, &[&bth_header, &leaf_records]);
        let second = heap_block(&[0, 0], &[&utf16("Spans two blocks")]);
        (first, second)
    }

    struct Fixture {
        _file: tempfile::NamedTempFile,
        reader: PstByteReader,
        bbt: BbtIndex,
    }

    fn fixture(first: &[u8], second: &[u8]) -> Fixture {
        let mut xblock = vec![0x01, 0x01];
        xblock.extend_from_slice(&2u16.to_le_bytes());
        xblock.extend_from_slice(&((first.len() + second.len()) as u32).to_le_bytes());
        xblock.extend_from_slice(&FIRST_BID.to_le_bytes());
        xblock.extend_from_slice(&SECOND_BID.to_le_bytes());

        // The header area covers the crypt-method byte, which stays 0 (none).
        let mut bytes = vec![0u8; 1024];
        bytes[10..12].copy_from_slice(&23u16.to_le_bytes());
        let mut entries = Vec::new();
        for (bid, payload) in [
            (XBLOCK_BID, xblock.as_slice()),
            (FIRST_BID, first),
            (SECOND_BID, second),
        ] {
            let offset = bytes.len() as u64;
            bytes.extend(crate::pst::payload::unicode_block_for_tests(payload));
            entries.push(BbtEntry {
                block_id: BlockId(bid),
                offset: ByteOffset(offset),
                size: payload.len() as u64,
            });
        }
        let file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(file.path(), &bytes).unwrap();
        let reader = PstByteReader::open(file.path()).unwrap();
        let bbt = BbtIndex {
            root: None,
            entries,
            parsed_pages: 0,
            discovered_child_pages: 0,
            traversal_error_count: 0,
            duplicate_entry_count: 0,
            truncated_entry_count: 0,
            status: "test".to_string(),
            lookup_index: Default::default(),
            page_limit_reached: false,
        };
        Fixture {
            _file: file,
            reader,
            bbt,
        }
    }

    fn read(fixture: &Fixture) -> PstdResult<(BthMap, usize)> {
        read_property_context_tree(
            TcSource {
                reader: &fixture.reader,
                bbt: &fixture.bbt,
                limits: ParserLimits::default(),
            },
            BlockId(XBLOCK_BID),
        )
    }

    #[test]
    fn reads_property_values_across_heap_blocks() {
        let mut records = leaf(0x0037, 0x001f, hid(1, 1));
        records.extend(leaf(0x0e08, 0x0003, 4242));
        let (first, second) = two_block_heap(records);
        let fixture = fixture(&first, &second);

        let (bth, blocks) = read(&fixture).unwrap();
        assert_eq!(blocks, 2);
        let context = PropertyContext::from_bth_with_fallback_charset(&bth, None)
            .unwrap()
            .context;
        assert_eq!(
            context.string_value(PR_SUBJECT).as_deref(),
            Some("Spans two blocks")
        );
        assert_eq!(
            bth.lookup(&PR_MESSAGE_SIZE.to_le_bytes()),
            Some(4242u32.to_le_bytes().as_slice())
        );
    }

    #[test]
    fn rejects_repeated_properties_and_non_pc_heaps() {
        let mut records = leaf(0x0037, 0x001f, hid(1, 1));
        records.extend(leaf(0x0037, 0x001f, hid(1, 1)));
        let (first, second) = two_block_heap(records);
        assert!(read(&fixture(&first, &second))
            .unwrap_err()
            .to_string()
            .contains("repeats property 0x0037"));

        let (mut first, second) = two_block_heap(leaf(0x0037, 0x001f, hid(1, 1)));
        first[3] = 0x7c;
        assert!(read(&fixture(&first, &second))
            .unwrap_err()
            .to_string()
            .contains("not a property-context heap"));
    }
}
