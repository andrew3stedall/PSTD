//! Resolve PC out-of-line values in the owning node's subnode namespace.
use std::collections::HashSet;

use crate::pst::bbt::BbtIndex;
use crate::pst::bth::BthMap;
use crate::pst::data_tree::load_attachment_data_payload;
use crate::pst::limits::ParserLimits;
use crate::pst::payload::{load_payload_block, PayloadBlock};
use crate::pst::primitives::BlockId;
use crate::pst::reader::PstByteReader;
use crate::pst::subnodes::unicode_subnode_entries;

pub(crate) fn resolve_values(
    bth: &mut BthMap,
    root: Option<BlockId>,
    reader: &PstByteReader,
    bbt: &BbtIndex,
    limits: ParserLimits,
) {
    for entry in &mut bth.entries {
        let Some(hnid) = entry.unresolved_hnid else {
            continue;
        };
        if hnid & 0x1f == 0 {
            continue;
        } // Invalid heap ID, not a subnode.
        let Some(root) = root else { continue };
        let Some(bid) = find_data_bid(root, hnid, reader, bbt, limits) else {
            continue;
        };
        if let Ok(payload) = load_attachment_data_payload(reader, bbt, bid, None, limits) {
            entry.value = payload.bytes;
            entry.unresolved_hnid = None;
        }
    }
}

/// Identify a PC's own namespace, never a sibling attachment's data tree.
pub(crate) fn owning_subnode_root(
    block: &PayloadBlock,
    blocks: &[PayloadBlock],
) -> Option<BlockId> {
    let owners: Vec<_> = blocks
        .iter()
        .filter_map(unicode_subnode_entries)
        .flatten()
        .filter(|entry| entry.data_block_id == block.block_id)
        .collect();
    if owners.len() == 1 {
        owners[0].subnode_block_id
    } else {
        None
    }
}

fn find_data_bid(
    root: BlockId,
    nid: u32,
    reader: &PstByteReader,
    bbt: &BbtIndex,
    limits: ParserLimits,
) -> Option<BlockId> {
    let version = reader.read_at(10, 2).ok()?;
    let width = if matches!(u16::from_le_bytes(version.try_into().ok()?), 14 | 15) {
        4
    } else {
        8
    };
    let header = if width == 4 { 4 } else { 8 };
    let mut pending = vec![(root, 0)];
    let mut seen = HashSet::new();
    let mut matches = Vec::new();
    while let Some((bid, depth)) = pending.pop() {
        if depth > limits.max_subnode_depth || !seen.insert(bid) {
            return None;
        }
        let payload = load_payload_block(reader, bbt, bid, limits).ok()?;
        let bytes = &payload.bytes;
        if bytes.len() < header || bytes[0] != 2 || bytes[1] > 1 {
            return None;
        }
        let level = bytes[1];
        let count = u16::from_le_bytes(bytes[2..4].try_into().ok()?) as usize;
        let stride = width * if level == 0 { 3 } else { 2 };
        if count == 0 || header + count * stride > bytes.len() {
            return None;
        }
        for index in 0..count {
            let start = header + index * stride;
            let number = |offset| -> Option<u64> {
                let field = bytes.get(offset..offset + width)?;
                Some(if width == 4 {
                    u32::from_le_bytes(field.try_into().ok()?) as u64
                } else {
                    u64::from_le_bytes(field.try_into().ok()?)
                })
            };
            let key = u32::try_from(number(start)?).ok()?;
            let data = BlockId(number(start + width)?);
            if data.0 == 0 {
                return None;
            }
            if level == 1 {
                pending.push((data, depth + 1));
            } else if key == nid {
                matches.push(data);
            }
        }
    }
    if matches.len() == 1 {
        Some(matches[0])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pst::bbt::BbtEntry;
    use crate::pst::bth::{BthEntry, BthHeader};
    use crate::pst::primitives::ByteOffset;

    fn fixture(blocks: &[(u64, Vec<u8>)]) -> (tempfile::NamedTempFile, BbtIndex) {
        let file = tempfile::NamedTempFile::new().unwrap();
        let mut bytes = vec![0; 1024];
        bytes[10..12].copy_from_slice(&23u16.to_le_bytes());
        let mut bbt = BbtIndex {
            root: None,
            entries: Vec::new(),
            parsed_pages: 0,
            discovered_child_pages: 0,
            traversal_error_count: 0,
            duplicate_entry_count: 0,
            truncated_entry_count: 0,
            status: "test".into(),
            lookup_index: Default::default(),
            page_limit_reached: false,
        };
        for (bid, payload) in blocks {
            let offset = bytes.len();
            bytes.extend(payload);
            bbt.entries.push(BbtEntry {
                block_id: BlockId(*bid),
                offset: ByteOffset(offset as u64),
                size: payload.len() as u64,
            });
        }
        std::fs::write(file.path(), bytes).unwrap();
        (file, bbt)
    }

    fn leaf(entries: &[(u32, u64)]) -> Vec<u8> {
        let mut bytes = vec![2, 0, entries.len() as u8, 0, 0, 0, 0, 0];
        for (nid, bid) in entries {
            bytes.extend((*nid as u64).to_le_bytes());
            bytes.extend(bid.to_le_bytes());
            bytes.extend(0u64.to_le_bytes());
        }
        bytes
    }

    fn map(hnid: u32) -> BthMap {
        BthMap {
            header: BthHeader {
                key_size: 4,
                value_size: 6,
                entry_count: 1,
                root_allocation: 0,
            },
            entries: vec![BthEntry {
                key: 0x1000001eu32.to_le_bytes().to_vec(),
                value: hnid.to_le_bytes().to_vec(),
                unresolved_hnid: Some(hnid),
            }],
        }
    }

    #[test]
    fn resolves_only_the_owning_namespace_and_rejects_missing_duplicate_and_bad_heap_ids() {
        let (file, bbt) = fixture(&[
            (0x102, leaf(&[(0x31f, 0x200)])),
            (0x106, leaf(&[(0x31f, 0x204)])),
            (0x10a, leaf(&[(0x31f, 0x200), (0x31f, 0x204)])),
            (0x200, b"owner".to_vec()),
            (0x204, b"sibling".to_vec()),
        ]);
        let reader = PstByteReader::open(file.path()).unwrap();
        let limits = ParserLimits::default();
        let mut bth = map(0x31f);
        resolve_values(&mut bth, Some(BlockId(0x102)), &reader, &bbt, limits);
        assert_eq!(bth.entries[0].value, b"owner");
        assert_eq!(bth.entries[0].unresolved_hnid, None);
        for (root, hnid) in [
            (None, 0x31f),
            (Some(BlockId(0x10a)), 0x31f),
            (Some(BlockId(0x102)), 0x33f),
            (Some(BlockId(0x102)), 0x20),
        ] {
            let mut bth = map(hnid);
            resolve_values(&mut bth, root, &reader, &bbt, limits);
            assert_eq!(bth.entries[0].unresolved_hnid, Some(hnid));
        }
    }

    #[test]
    fn resolves_subnode_index_and_rejects_cycle_or_truncated_stream() {
        let mut index = vec![2, 1, 1, 0, 0, 0, 0, 0];
        index.extend(0x31fu64.to_le_bytes());
        index.extend(0x106u64.to_le_bytes());
        let mut cycle = index.clone();
        cycle[16..24].copy_from_slice(&0x10au64.to_le_bytes());
        let truncated_tree = vec![1, 1, 1, 0, 100, 0, 0, 0];
        let (file, bbt) = fixture(&[
            (0x102, index),
            (0x106, leaf(&[(0x31f, 0x200)])),
            (0x10a, cycle),
            (0x10e, leaf(&[(0x31f, 0x202)])),
            (0x200, b"indexed".to_vec()),
            (0x202, truncated_tree),
        ]);
        let reader = PstByteReader::open(file.path()).unwrap();
        let mut bth = map(0x31f);
        resolve_values(
            &mut bth,
            Some(BlockId(0x102)),
            &reader,
            &bbt,
            ParserLimits::default(),
        );
        assert_eq!(bth.entries[0].value, b"indexed");
        for root in [0x10a, 0x10e] {
            let mut bth = map(0x31f);
            resolve_values(
                &mut bth,
                Some(BlockId(root)),
                &reader,
                &bbt,
                ParserLimits::default(),
            );
            assert_eq!(bth.entries[0].unresolved_hnid, Some(0x31f));
        }
    }
}
