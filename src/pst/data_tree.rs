use std::collections::HashSet;

use crate::error::{PstdError, PstdResult};
use crate::pst::bbt::BbtIndex;
use crate::pst::limits::ParserLimits;
use crate::pst::payload::load_payload_block;
use crate::pst::primitives::BlockId;
use crate::pst::reader::PstByteReader;

const BID_INTERNAL_MASK: u64 = 0x02;
const XBLOCK_DIRECT_HEADER: u16 = 0x0101;
const XBLOCK_INDIRECT_HEADER: u16 = 0x0201;
const XBLOCK_HEADER_BYTES: usize = 8;
const BID_WIDTHS: &[usize] = &[8, 4];
const DOCX_ZIP_SIGNATURE: &[u8; 4] = b"PK\x03\x04";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataTreePayload {
    pub root_bid: BlockId,
    pub child_bids: Vec<BlockId>,
    pub declared_total_bytes: u64,
    pub bytes: Vec<u8>,
    pub status: String,
}

/// Resolve an attachment data BID using the same direct/indirect block model
/// used by libpst's `pst_attach_to_file` path.
///
/// The resolver deliberately accepts arbitrary payload bytes.  A data tree is
/// not inherently a DOCX, and using a format-specific signature here caused
/// otherwise valid PDFs, images, archives, and OLE payloads to be discarded.
pub fn load_attachment_data_payload(
    reader: &PstByteReader,
    bbt: &BbtIndex,
    root_bid: BlockId,
    expected_size: Option<u64>,
    limits: ParserLimits,
) -> PstdResult<DataTreePayload> {
    if root_bid.0 & BID_INTERNAL_MASK == 0 {
        let payload = load_payload_block(reader, bbt, root_bid, limits)?;
        let actual_size = payload.bytes.len() as u64;
        let expected_status = match expected_size {
            Some(expected) if expected == actual_size => "expected_size_matched",
            Some(_) => "expected_size_differs",
            None => "expected_size_absent",
        };
        return Ok(DataTreePayload {
            root_bid,
            child_bids: vec![root_bid],
            declared_total_bytes: actual_size,
            bytes: payload.bytes,
            status: format!(
                "direct_payload_block_loaded; root_bid=0x{:x}; total_bytes={actual_size}; {expected_status}",
                root_bid.0
            ),
        });
    }

    let root = load_payload_block(reader, bbt, root_bid, limits)?;
    let (header, child_count, declared_total_bytes) = parse_xblock_header(&root.bytes, root_bid)?;
    if child_count == 0 {
        return Err(PstdError::pst_parse(
            Some(root.block_ref.offset.0),
            "attachment data tree contains no child BIDs",
        ));
    }
    if declared_total_bytes > limits.max_block_bytes {
        return Err(PstdError::pst_read(
            Some(root.block_ref.offset.0),
            format!(
                "attachment data tree total {declared_total_bytes} exceeds configured limit {}",
                limits.max_block_bytes
            ),
        ));
    }

    let bid_width = child_bid_width(&root.bytes, child_count, bbt).ok_or_else(|| {
        PstdError::pst_parse(
            Some(root.block_ref.offset.0),
            format!(
                "attachment data tree declares {child_count} child BIDs but the child array is truncated or references unknown blocks"
            ),
        )
    })?;
    let child_bytes = child_count.checked_mul(bid_width).ok_or_else(|| {
        PstdError::pst_parse(
            Some(root.block_ref.offset.0),
            "attachment data tree child array overflow",
        )
    })?;
    let child_end = XBLOCK_HEADER_BYTES
        .checked_add(child_bytes)
        .ok_or_else(|| {
            PstdError::pst_parse(
                Some(root.block_ref.offset.0),
                "attachment data tree length overflow",
            )
        })?;
    if child_end > root.bytes.len() {
        return Err(PstdError::pst_parse(
            Some(root.block_ref.offset.0),
            format!(
                "attachment data tree declares {child_count} child BIDs but only {} bytes are available",
                root.bytes.len().saturating_sub(XBLOCK_HEADER_BYTES)
            ),
        ));
    }

    let mut seen = HashSet::new();
    seen.insert(root_bid);
    let mut child_bids = Vec::new();
    let mut bytes = Vec::new();
    for index in 0..child_count {
        let start = XBLOCK_HEADER_BYTES + index * bid_width;
        let child_bid = read_child_bid(&root.bytes[start..start + bid_width], bid_width);
        if child_bid.0 == 0 {
            return Err(PstdError::pst_parse(
                Some(root.block_ref.offset.0),
                format!("attachment data tree child {index} has a zero BID"),
            ));
        }
        if !seen.insert(child_bid) {
            return Err(PstdError::pst_parse(
                Some(root.block_ref.offset.0),
                format!("attachment data tree repeats child BID 0x{:x}", child_bid.0),
            ));
        }
        append_attachment_data(
            reader,
            bbt,
            child_bid,
            1,
            header,
            declared_total_bytes,
            limits,
            &mut seen,
            &mut child_bids,
            &mut bytes,
        )?;
    }

    if bytes.len() as u64 != declared_total_bytes {
        return Err(PstdError::pst_parse(
            Some(root.block_ref.offset.0),
            format!(
                "attachment data tree resolved {} bytes but declared {declared_total_bytes}",
                bytes.len()
            ),
        ));
    }
    let expected_status = match expected_size {
        Some(expected) if expected == declared_total_bytes => "metadata_size_matched",
        Some(_) => "metadata_size_differs_from_data_tree_total",
        None => "metadata_size_absent",
    };

    let child_block_count = child_bids.len();
    Ok(DataTreePayload {
        root_bid,
        child_bids,
        declared_total_bytes,
        bytes,
        status: format!(
            "attachment_data_tree_loaded; root_bid=0x{:x}; header=0x{header:04x}; child_blocks={}; total_bytes={declared_total_bytes}; {expected_status}",
            root_bid.0,
            child_block_count
        ),
    })
}

fn append_attachment_data(
    reader: &PstByteReader,
    bbt: &BbtIndex,
    block_id: BlockId,
    depth: usize,
    parent_header: u16,
    declared_total_bytes: u64,
    limits: ParserLimits,
    seen: &mut HashSet<BlockId>,
    child_bids: &mut Vec<BlockId>,
    bytes: &mut Vec<u8>,
) -> PstdResult<()> {
    if depth > limits.max_subnode_depth {
        return Err(PstdError::pst_read(
            None,
            format!(
                "attachment data tree depth {depth} exceeds configured limit {}",
                limits.max_subnode_depth
            ),
        ));
    }

    if parent_header == XBLOCK_DIRECT_HEADER && block_id.0 & BID_INTERNAL_MASK != 0 {
        return Err(PstdError::pst_parse(
            None,
            format!(
                "invalid external BID 0x{:x} in direct attachment data tree",
                block_id.0
            ),
        ));
    }

    let payload = load_payload_block(reader, bbt, block_id, limits)?;
    if block_id.0 & BID_INTERNAL_MASK != 0 {
        let (header, child_count, nested_total) = parse_xblock_header(&payload.bytes, block_id)?;
        if parent_header == XBLOCK_DIRECT_HEADER && header != XBLOCK_DIRECT_HEADER {
            return Err(PstdError::pst_parse(
                Some(payload.block_ref.offset.0),
                format!(
                    "direct attachment data tree child 0x{:x} has nested header 0x{header:04x}",
                    block_id.0
                ),
            ));
        }
        if nested_total > declared_total_bytes {
            return Err(PstdError::pst_parse(
                Some(payload.block_ref.offset.0),
                format!(
                    "attachment data tree child total {nested_total} exceeds root total {declared_total_bytes}"
                ),
            ));
        }
        let bid_width = child_bid_width(&payload.bytes, child_count, bbt).ok_or_else(|| {
            PstdError::pst_parse(
                Some(payload.block_ref.offset.0),
                "nested attachment data tree child BID array is truncated or references unknown blocks",
            )
        })?;
        let child_bytes = child_count.checked_mul(bid_width).ok_or_else(|| {
            PstdError::pst_parse(
                Some(payload.block_ref.offset.0),
                "nested attachment data tree child array overflow",
            )
        })?;
        let child_end = XBLOCK_HEADER_BYTES
            .checked_add(child_bytes)
            .ok_or_else(|| {
                PstdError::pst_parse(
                    Some(payload.block_ref.offset.0),
                    "nested attachment data tree length overflow",
                )
            })?;
        if child_end > payload.bytes.len() {
            return Err(PstdError::pst_parse(
                Some(payload.block_ref.offset.0),
                format!(
                    "nested attachment data tree declares {child_count} child BIDs but only {} bytes are available",
                    payload.bytes.len().saturating_sub(XBLOCK_HEADER_BYTES)
                ),
            ));
        }
        for index in 0..child_count {
            let start = XBLOCK_HEADER_BYTES + index * bid_width;
            let child_id = read_child_bid(&payload.bytes[start..start + bid_width], bid_width);
            if child_id.0 == 0 || !seen.insert(child_id) {
                return Err(PstdError::pst_parse(
                    Some(payload.block_ref.offset.0),
                    format!(
                        "nested attachment data tree child {index} has an invalid or repeated BID 0x{:x}",
                        child_id.0
                    ),
                ));
            }
            append_attachment_data(
                reader,
                bbt,
                child_id,
                depth + 1,
                header,
                declared_total_bytes,
                limits,
                seen,
                child_bids,
                bytes,
            )?;
        }
        return Ok(());
    }

    let next_len = bytes
        .len()
        .checked_add(payload.bytes.len())
        .ok_or_else(|| {
            PstdError::pst_parse(
                Some(payload.block_ref.offset.0),
                "attachment payload length overflow",
            )
        })?;
    if next_len as u64 > declared_total_bytes {
        return Err(PstdError::pst_parse(
            Some(payload.block_ref.offset.0),
            format!("attachment payload exceeds declared total {declared_total_bytes} bytes"),
        ));
    }
    bytes.extend_from_slice(&payload.bytes);
    child_bids.push(block_id);
    Ok(())
}

fn child_bid_width(bytes: &[u8], child_count: usize, bbt: &BbtIndex) -> Option<usize> {
    BID_WIDTHS.iter().copied().find(|width| {
        let Some(child_bytes) = child_count.checked_mul(*width) else {
            return false;
        };
        let Some(end) = XBLOCK_HEADER_BYTES.checked_add(child_bytes) else {
            return false;
        };
        end <= bytes.len()
            && (0..child_count).all(|index| {
                let start = XBLOCK_HEADER_BYTES + index * *width;
                let child_bid = read_child_bid(&bytes[start..start + *width], *width);
                child_bid.0 != 0 && bbt.lookup(child_bid).is_some()
            })
    })
}

fn read_child_bid(bytes: &[u8], width: usize) -> BlockId {
    match width {
        4 => BlockId(
            u32::from_le_bytes(bytes.try_into().expect("four-byte attachment data BID")) as u64,
        ),
        8 => BlockId(u64::from_le_bytes(
            bytes.try_into().expect("eight-byte attachment data BID"),
        )),
        _ => unreachable!("unsupported attachment data BID width"),
    }
}

fn parse_xblock_header(bytes: &[u8], block_id: BlockId) -> PstdResult<(u16, usize, u64)> {
    if bytes.len() < XBLOCK_HEADER_BYTES {
        return Err(PstdError::pst_parse(
            None,
            format!(
                "attachment data tree block 0x{:x} header is truncated",
                block_id.0
            ),
        ));
    }
    let header = u16::from_le_bytes([bytes[0], bytes[1]]);
    if !matches!(header, XBLOCK_DIRECT_HEADER | XBLOCK_INDIRECT_HEADER) {
        return Err(PstdError::pst_parse(
            None,
            format!(
                "unsupported attachment data tree header 0x{header:04x} in block 0x{:x}",
                block_id.0
            ),
        ));
    }
    let child_count = u16::from_le_bytes([bytes[2], bytes[3]]) as usize;
    let declared_total_bytes =
        u32::from_le_bytes(bytes[4..8].try_into().expect("four-byte data tree total")) as u64;
    Ok((header, child_count, declared_total_bytes))
}

pub fn load_unicode_xblock_payload(
    reader: &PstByteReader,
    bbt: &BbtIndex,
    root_bid: BlockId,
    expected_size: u64,
    limits: ParserLimits,
) -> PstdResult<DataTreePayload> {
    let payload = load_attachment_data_payload(reader, bbt, root_bid, Some(expected_size), limits)?;
    if !payload.bytes.starts_with(DOCX_ZIP_SIGNATURE) {
        return Err(PstdError::pst_parse(
            None,
            format!(
                "resolved attachment payload has unexpected signature {}",
                hex::encode(&payload.bytes[..payload.bytes.len().min(4)])
            ),
        ));
    }

    let status = payload.status.replace(
        "metadata_size_differs_from_data_tree_total",
        "metadata_size_differs_from_xblock_total",
    );
    Ok(DataTreePayload {
        status: format!("{status}; zip_signature=504b0304"),
        ..payload
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::NamedTempFile;

    use super::{load_attachment_data_payload, load_unicode_xblock_payload};
    use crate::pst::bbt::{BbtEntry, BbtIndex};
    use crate::pst::limits::ParserLimits;
    use crate::pst::primitives::{BlockId, ByteOffset};
    use crate::pst::reader::PstByteReader;

    #[test]
    fn loads_ordered_unicode_xblock_payload() {
        let first = b"PK\x03\x04first";
        let second = b"second";
        let root = xblock(&[0x640, 0x644], (first.len() + second.len()) as u32);
        let (file, bbt) = fixture(&[
            (0x632, root),
            (0x640, first.to_vec()),
            (0x644, second.to_vec()),
        ]);
        let reader = PstByteReader::open(file.path()).unwrap();

        let payload = load_unicode_xblock_payload(
            &reader,
            &bbt,
            BlockId(0x632),
            (first.len() + second.len()) as u64,
            ParserLimits::default(),
        )
        .unwrap();

        assert_eq!(payload.child_bids, vec![BlockId(0x640), BlockId(0x644)]);
        assert_eq!(
            payload.bytes,
            [first.as_slice(), second.as_slice()].concat()
        );
        assert!(payload.status.contains("child_blocks=2"));
    }

    #[test]
    fn preserves_metadata_size_difference_and_rejects_non_docx_payload() {
        let root = xblock(&[0x640], 4);
        let (file, bbt) = fixture(&[(0x632, root), (0x640, b"PK\x03\x04".to_vec())]);
        let reader = PstByteReader::open(file.path()).unwrap();

        let mismatch =
            load_unicode_xblock_payload(&reader, &bbt, BlockId(0x632), 5, ParserLimits::default())
                .unwrap();
        assert_eq!(mismatch.declared_total_bytes, 4);
        assert!(mismatch
            .status
            .contains("metadata_size_differs_from_xblock_total"));

        let root = xblock(&[0x640], 4);
        let (file, bbt) = fixture(&[(0x632, root), (0x640, b"nope".to_vec())]);
        let reader = PstByteReader::open(file.path()).unwrap();
        let signature =
            load_unicode_xblock_payload(&reader, &bbt, BlockId(0x632), 4, ParserLimits::default())
                .unwrap_err();
        assert!(signature.to_string().contains("unexpected signature"));
    }

    #[test]
    fn loads_arbitrary_direct_attachment_payload_without_format_assumptions() {
        let bytes = b"%PDF-1.7\nnot a docx".to_vec();
        let (file, bbt) = fixture(&[(0x640, bytes.clone())]);
        let reader = PstByteReader::open(file.path()).unwrap();

        let payload = load_attachment_data_payload(
            &reader,
            &bbt,
            BlockId(0x640),
            Some(bytes.len() as u64),
            ParserLimits::default(),
        )
        .unwrap();

        assert_eq!(payload.bytes, bytes);
        assert_eq!(payload.child_bids, vec![BlockId(0x640)]);
        assert!(payload.status.contains("direct_payload_block_loaded"));
    }

    #[test]
    fn loads_indirect_attachment_data_tree() {
        let first = b"OLE-";
        let second = b"payload";
        let root = xblock_with_header(0x0201, &[0x642], (first.len() + second.len()) as u32);
        let nested =
            xblock_with_header(0x0101, &[0x640, 0x644], (first.len() + second.len()) as u32);
        let (file, bbt) = fixture(&[
            (0x632, root),
            (0x642, nested),
            (0x640, first.to_vec()),
            (0x644, second.to_vec()),
        ]);
        let reader = PstByteReader::open(file.path()).unwrap();

        let payload = load_attachment_data_payload(
            &reader,
            &bbt,
            BlockId(0x632),
            None,
            ParserLimits::default(),
        )
        .unwrap();

        assert_eq!(
            payload.bytes,
            [first.as_slice(), second.as_slice()].concat()
        );
        assert_eq!(payload.child_bids, vec![BlockId(0x640), BlockId(0x644)]);
        assert!(payload.status.contains("header=0x0201"));
    }

    #[test]
    fn rejects_truncated_or_internal_children() {
        let mut truncated = xblock(&[0x640], 4);
        truncated.truncate(12);
        truncated[2..4].copy_from_slice(&2u16.to_le_bytes());
        let (file, bbt) = fixture(&[(0x632, truncated), (0x640, b"PK\x03\x04".to_vec())]);
        let reader = PstByteReader::open(file.path()).unwrap();
        let error =
            load_unicode_xblock_payload(&reader, &bbt, BlockId(0x632), 4, ParserLimits::default())
                .unwrap_err();
        assert!(error.to_string().contains("declares 2 child BIDs"));

        let root = xblock(&[0x642], 4);
        let (file, bbt) = fixture(&[(0x632, root), (0x642, b"PK\x03\x04".to_vec())]);
        let reader = PstByteReader::open(file.path()).unwrap();
        let error =
            load_unicode_xblock_payload(&reader, &bbt, BlockId(0x632), 4, ParserLimits::default())
                .unwrap_err();
        assert!(error.to_string().contains("invalid external BID"));
    }

    fn xblock(child_bids: &[u64], total: u32) -> Vec<u8> {
        xblock_with_header(0x0101, child_bids, total)
    }

    fn xblock_with_header(header: u16, child_bids: &[u64], total: u32) -> Vec<u8> {
        let mut bytes = vec![0; 8 + child_bids.len() * 8];
        bytes[0..2].copy_from_slice(&header.to_le_bytes());
        bytes[2..4].copy_from_slice(&(child_bids.len() as u16).to_le_bytes());
        bytes[4..8].copy_from_slice(&total.to_le_bytes());
        for (index, bid) in child_bids.iter().enumerate() {
            let start = 8 + index * 8;
            bytes[start..start + 8].copy_from_slice(&bid.to_le_bytes());
        }
        bytes
    }

    fn fixture(blocks: &[(u64, Vec<u8>)]) -> (NamedTempFile, BbtIndex) {
        let file = NamedTempFile::new().unwrap();
        let mut file_bytes = vec![0; 1024];
        let mut entries = Vec::new();
        let mut offset = 600usize;
        for (bid, bytes) in blocks {
            if file_bytes.len() < offset + bytes.len() {
                file_bytes.resize(offset + bytes.len(), 0);
            }
            file_bytes[offset..offset + bytes.len()].copy_from_slice(bytes);
            entries.push(BbtEntry {
                block_id: BlockId(*bid),
                offset: ByteOffset(offset as u64),
                size: bytes.len() as u64,
            });
            offset += bytes.len() + 32;
        }
        fs::write(file.path(), file_bytes).unwrap();
        (
            file,
            BbtIndex {
                root: None,
                entries,
                parsed_pages: 0,
                discovered_child_pages: 0,
                traversal_error_count: 0,
                duplicate_entry_count: 0,
                truncated_entry_count: 0,
                status: "test".to_string(),
            },
        )
    }
}
