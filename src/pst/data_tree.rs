use std::collections::HashSet;

use crate::error::{PstdError, PstdResult};
use crate::pst::bbt::BbtIndex;
use crate::pst::limits::ParserLimits;
use crate::pst::payload::load_payload_block;
use crate::pst::primitives::BlockId;
use crate::pst::reader::PstByteReader;

const BID_INTERNAL_MASK: u64 = 0x02;
const XBLOCK_TYPE: u8 = 0x01;
const XBLOCK_LEVEL: u8 = 0x01;
const XBLOCK_HEADER_BYTES: usize = 8;
const UNICODE_BID_BYTES: usize = 8;
const DOCX_ZIP_SIGNATURE: &[u8; 4] = b"PK\x03\x04";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataTreePayload {
    pub root_bid: BlockId,
    pub child_bids: Vec<BlockId>,
    pub declared_total_bytes: u64,
    pub bytes: Vec<u8>,
    pub status: String,
}

pub fn load_unicode_xblock_payload(
    reader: &PstByteReader,
    bbt: &BbtIndex,
    root_bid: BlockId,
    expected_size: u64,
    limits: ParserLimits,
) -> PstdResult<DataTreePayload> {
    if root_bid.0 & BID_INTERNAL_MASK == 0 {
        return Err(PstdError::pst_parse(
            None,
            format!("data-tree root BID 0x{:x} is not internal", root_bid.0),
        ));
    }

    let root = load_payload_block(reader, bbt, root_bid, limits)?;
    if root.bytes.len() < XBLOCK_HEADER_BYTES {
        return Err(PstdError::pst_parse(
            Some(root.block_ref.offset.0),
            "XBLOCK header is truncated",
        ));
    }
    if root.bytes[0] != XBLOCK_TYPE || root.bytes[1] != XBLOCK_LEVEL {
        return Err(PstdError::pst_parse(
            Some(root.block_ref.offset.0),
            format!(
                "unsupported data-tree root type 0x{:02x} level 0x{:02x}",
                root.bytes[0], root.bytes[1]
            ),
        ));
    }

    let child_count = u16::from_le_bytes([root.bytes[2], root.bytes[3]]) as usize;
    let declared_total_bytes =
        u32::from_le_bytes(root.bytes[4..8].try_into().expect("four-byte XBLOCK total")) as u64;
    if child_count == 0 {
        return Err(PstdError::pst_parse(
            Some(root.block_ref.offset.0),
            "XBLOCK contains no child BIDs",
        ));
    }
    if declared_total_bytes != expected_size {
        return Err(PstdError::pst_parse(
            Some(root.block_ref.offset.0),
            format!(
                "XBLOCK declared total {declared_total_bytes} does not match expected attachment size {expected_size}"
            ),
        ));
    }
    if declared_total_bytes > limits.max_block_bytes {
        return Err(PstdError::pst_read(
            Some(root.block_ref.offset.0),
            format!(
                "XBLOCK total {declared_total_bytes} exceeds configured limit {}",
                limits.max_block_bytes
            ),
        ));
    }

    let child_bytes = child_count
        .checked_mul(UNICODE_BID_BYTES)
        .ok_or_else(|| PstdError::pst_parse(Some(root.block_ref.offset.0), "XBLOCK child array overflow"))?;
    let child_end = XBLOCK_HEADER_BYTES
        .checked_add(child_bytes)
        .ok_or_else(|| PstdError::pst_parse(Some(root.block_ref.offset.0), "XBLOCK length overflow"))?;
    if child_end > root.bytes.len() {
        return Err(PstdError::pst_parse(
            Some(root.block_ref.offset.0),
            format!(
                "XBLOCK declares {child_count} child BIDs but only {} bytes are available",
                root.bytes.len().saturating_sub(XBLOCK_HEADER_BYTES)
            ),
        ));
    }

    let mut seen = HashSet::new();
    let mut child_bids = Vec::with_capacity(child_count);
    let mut bytes = Vec::with_capacity(declared_total_bytes as usize);
    for index in 0..child_count {
        let start = XBLOCK_HEADER_BYTES + index * UNICODE_BID_BYTES;
        let child_bid = BlockId(u64::from_le_bytes(
            root.bytes[start..start + UNICODE_BID_BYTES]
                .try_into()
                .expect("eight-byte Unicode BID"),
        ));
        if child_bid.0 == 0 || child_bid.0 & BID_INTERNAL_MASK != 0 {
            return Err(PstdError::pst_parse(
                Some(root.block_ref.offset.0),
                format!("XBLOCK child {index} has invalid external BID 0x{:x}", child_bid.0),
            ));
        }
        if !seen.insert(child_bid) {
            return Err(PstdError::pst_parse(
                Some(root.block_ref.offset.0),
                format!("XBLOCK repeats child BID 0x{:x}", child_bid.0),
            ));
        }

        let child = load_payload_block(reader, bbt, child_bid, limits)?;
        let next_len = bytes
            .len()
            .checked_add(child.bytes.len())
            .ok_or_else(|| PstdError::pst_parse(Some(child.block_ref.offset.0), "XBLOCK payload length overflow"))?;
        if next_len as u64 > declared_total_bytes {
            return Err(PstdError::pst_parse(
                Some(child.block_ref.offset.0),
                format!(
                    "XBLOCK child data exceeds declared total {declared_total_bytes} bytes"
                ),
            ));
        }
        bytes.extend_from_slice(&child.bytes);
        child_bids.push(child_bid);
    }

    if bytes.len() as u64 != declared_total_bytes {
        return Err(PstdError::pst_parse(
            Some(root.block_ref.offset.0),
            format!(
                "XBLOCK resolved {} bytes but declared {declared_total_bytes}",
                bytes.len()
            ),
        ));
    }
    if !bytes.starts_with(DOCX_ZIP_SIGNATURE) {
        return Err(PstdError::pst_parse(
            Some(root.block_ref.offset.0),
            format!(
                "resolved attachment payload has unexpected signature {}",
                hex::encode(&bytes[..bytes.len().min(4)])
            ),
        ));
    }

    Ok(DataTreePayload {
        root_bid,
        child_bids: child_bids.clone(),
        declared_total_bytes,
        bytes,
        status: format!(
            "unicode_xblock_payload_loaded; root_bid=0x{:x}; child_blocks={}; total_bytes={declared_total_bytes}; zip_signature=504b0304",
            root_bid.0,
            child_bids.len()
        ),
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::NamedTempFile;

    use super::load_unicode_xblock_payload;
    use crate::pst::bbt::{BbtEntry, BbtIndex};
    use crate::pst::limits::ParserLimits;
    use crate::pst::primitives::{BlockId, ByteOffset};
    use crate::pst::reader::PstByteReader;

    #[test]
    fn loads_ordered_unicode_xblock_payload() {
        let first = b"PK\x03\x04first";
        let second = b"second";
        let root = xblock(&[0x640, 0x644], (first.len() + second.len()) as u32);
        let (file, bbt) = fixture(&[(0x632, root), (0x640, first.to_vec()), (0x644, second.to_vec())]);
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
        assert_eq!(payload.bytes, [first.as_slice(), second.as_slice()].concat());
        assert!(payload.status.contains("child_blocks=2"));
    }

    #[test]
    fn rejects_size_mismatch_and_non_docx_payload() {
        let root = xblock(&[0x640], 4);
        let (file, bbt) = fixture(&[(0x632, root), (0x640, b"nope".to_vec())]);
        let reader = PstByteReader::open(file.path()).unwrap();

        let mismatch = load_unicode_xblock_payload(
            &reader,
            &bbt,
            BlockId(0x632),
            5,
            ParserLimits::default(),
        )
        .unwrap_err();
        assert!(mismatch.to_string().contains("does not match expected"));

        let signature = load_unicode_xblock_payload(
            &reader,
            &bbt,
            BlockId(0x632),
            4,
            ParserLimits::default(),
        )
        .unwrap_err();
        assert!(signature.to_string().contains("unexpected signature"));
    }

    #[test]
    fn rejects_truncated_or_internal_children() {
        let mut truncated = xblock(&[0x640], 4);
        truncated.truncate(12);
        truncated[2..4].copy_from_slice(&2u16.to_le_bytes());
        let (file, bbt) = fixture(&[(0x632, truncated), (0x640, b"PK\x03\x04".to_vec())]);
        let reader = PstByteReader::open(file.path()).unwrap();
        let error = load_unicode_xblock_payload(
            &reader,
            &bbt,
            BlockId(0x632),
            4,
            ParserLimits::default(),
        )
        .unwrap_err();
        assert!(error.to_string().contains("declares 2 child BIDs"));

        let root = xblock(&[0x642], 4);
        let (file, bbt) = fixture(&[(0x632, root), (0x642, b"PK\x03\x04".to_vec())]);
        let reader = PstByteReader::open(file.path()).unwrap();
        let error = load_unicode_xblock_payload(
            &reader,
            &bbt,
            BlockId(0x632),
            4,
            ParserLimits::default(),
        )
        .unwrap_err();
        assert!(error.to_string().contains("invalid external BID"));
    }

    fn xblock(child_bids: &[u64], total: u32) -> Vec<u8> {
        let mut bytes = vec![0; 8 + child_bids.len() * 8];
        bytes[0] = 0x01;
        bytes[1] = 0x01;
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
