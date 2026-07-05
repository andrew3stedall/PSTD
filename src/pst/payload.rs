use crate::error::{PstdError, PstdResult};
use crate::pst::bbt::BbtIndex;
use crate::pst::block::{load_block, LoadedBlock};
use crate::pst::limits::ParserLimits;
use crate::pst::primitives::{BlockId, BlockRef};
use crate::pst::reader::PstByteReader;

#[derive(Debug, Clone)]
pub struct PayloadBlock {
    pub block_id: BlockId,
    pub block_ref: BlockRef,
    pub bytes: Vec<u8>,
    pub status: String,
}

pub fn load_payload_block(
    reader: &PstByteReader,
    bbt: &BbtIndex,
    block_id: BlockId,
    limits: ParserLimits,
) -> PstdResult<PayloadBlock> {
    let block_ref = bbt.lookup(block_id).ok_or_else(|| {
        PstdError::pst_parse(
            None,
            format!("payload block {:?} not found in BBT", block_id),
        )
    })?;

    if block_ref.size > limits.max_block_bytes {
        return Err(PstdError::pst_read(
            Some(block_ref.offset.0),
            format!(
                "payload block size {} exceeds configured limit {}",
                block_ref.size, limits.max_block_bytes
            ),
        ));
    }

    let LoadedBlock { bytes, .. } = load_block(reader, block_ref)?;
    Ok(PayloadBlock {
        block_id,
        block_ref,
        bytes,
        status: "payload_loaded".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::NamedTempFile;

    use super::load_payload_block;
    use crate::pst::bbt::{BbtEntry, BbtIndex};
    use crate::pst::limits::ParserLimits;
    use crate::pst::primitives::{BlockId, ByteOffset};
    use crate::pst::reader::PstByteReader;

    #[test]
    fn loads_payload_block_from_bbt_lookup() {
        let file = NamedTempFile::new().unwrap();
        fs::write(file.path(), b"0123456789payload").unwrap();
        let reader = PstByteReader::open(file.path()).unwrap();
        let bbt = index_with_entry(BlockId(42), 10, 7);

        let payload =
            load_payload_block(&reader, &bbt, BlockId(42), ParserLimits::default()).unwrap();
        assert_eq!(payload.block_id.0, 42);
        assert_eq!(payload.bytes, b"payload");
        assert_eq!(payload.status, "payload_loaded");
    }

    #[test]
    fn rejects_payload_blocks_over_limit() {
        let file = NamedTempFile::new().unwrap();
        fs::write(file.path(), b"0123456789payload").unwrap();
        let reader = PstByteReader::open(file.path()).unwrap();
        let bbt = index_with_entry(BlockId(42), 10, 7);
        let limits = ParserLimits {
            max_block_bytes: 3,
            ..ParserLimits::default()
        };

        let err = load_payload_block(&reader, &bbt, BlockId(42), limits).unwrap_err();
        assert!(err.to_string().contains("exceeds configured limit"));
    }

    fn index_with_entry(block_id: BlockId, offset: u64, size: u64) -> BbtIndex {
        BbtIndex {
            root: None,
            entries: vec![BbtEntry {
                block_id,
                offset: ByteOffset(offset),
                size,
            }],
            parsed_pages: 0,
            discovered_child_pages: 0,
            traversal_error_count: 0,
            duplicate_entry_count: 0,
            truncated_entry_count: 0,
            page_diagnostics: Vec::new(),
            status: "test".to_string(),
        }
    }
}
