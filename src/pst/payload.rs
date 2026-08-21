use crate::error::{PstdError, PstdResult};
use crate::pst::bbt::BbtIndex;
use crate::pst::block::{load_block, LoadedBlock};
use crate::pst::crypto::{decode_in_place, NDB_CRYPT_NONE};
use crate::pst::header::{
    PST_ANSI_HEADER_CRYPT_METHOD_OFFSET, PST_HEADER_CRYPT_METHOD_OFFSET, PST_INDEX_TYPE_ANSI32,
    PST_INDEX_TYPE_ANSI32A, PST_INDEX_TYPE_OFFSET,
};
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

const BID_INTERNAL_MASK: u64 = 0x02;

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

    let LoadedBlock { mut bytes, .. } = load_block(reader, block_ref)?;
    let internal = block_id.0 & BID_INTERNAL_MASK != 0;
    let crypt_method = read_crypt_method(reader)?;
    let status = if internal {
        "payload_loaded_internal".to_string()
    } else {
        decode_in_place(block_id.0, &mut bytes, crypt_method)
            .map_err(|error| PstdError::pst_parse(Some(block_ref.offset.0), error.to_string()))?
            .to_string()
    };
    Ok(PayloadBlock {
        block_id,
        block_ref,
        bytes,
        status: status.to_string(),
    })
}

fn read_crypt_method(reader: &PstByteReader) -> PstdResult<u8> {
    let index_type = if reader.file_size() > PST_INDEX_TYPE_OFFSET as u64 {
        reader.read_at(PST_INDEX_TYPE_OFFSET as u64, 1)?[0]
    } else {
        return Ok(NDB_CRYPT_NONE);
    };
    let offset = if matches!(index_type, PST_INDEX_TYPE_ANSI32 | PST_INDEX_TYPE_ANSI32A) {
        PST_ANSI_HEADER_CRYPT_METHOD_OFFSET
    } else {
        PST_HEADER_CRYPT_METHOD_OFFSET
    };
    if reader.file_size() <= offset as u64 {
        return Ok(NDB_CRYPT_NONE);
    }
    Ok(reader.read_at(offset as u64, 1)?[0])
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::NamedTempFile;

    use super::load_payload_block;
    use crate::pst::bbt::{BbtEntry, BbtIndex};
    use crate::pst::crypto::{encode_permutative, NDB_CRYPT_STRONG};
    use crate::pst::header::PST_HEADER_CRYPT_METHOD_OFFSET;
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
        assert_eq!(payload.status, "payload_loaded_internal");
    }

    #[test]
    fn decodes_permutatively_encoded_external_payload() {
        let plaintext = b"decoded payload";
        let encoded = encode_permutative(plaintext);
        let mut file_bytes = vec![0u8; 600];
        file_bytes[PST_HEADER_CRYPT_METHOD_OFFSET] = crate::pst::crypto::NDB_CRYPT_PERMUTE;
        file_bytes[520..520 + encoded.len()].copy_from_slice(&encoded);
        let file = NamedTempFile::new().unwrap();
        fs::write(file.path(), file_bytes).unwrap();
        let reader = PstByteReader::open(file.path()).unwrap();
        let bbt = index_with_entry(BlockId(0x7c), 520, encoded.len() as u64);

        let payload =
            load_payload_block(&reader, &bbt, BlockId(0x7c), ParserLimits::default()).unwrap();

        assert_eq!(payload.bytes, plaintext);
        assert_eq!(payload.status, "payload_loaded_permute_decoded");
    }

    #[test]
    fn decodes_strongly_encrypted_external_payload_in_production_loader() {
        let mut file_bytes = vec![0u8; 600];
        file_bytes[PST_HEADER_CRYPT_METHOD_OFFSET] = NDB_CRYPT_STRONG;
        file_bytes[520..537].copy_from_slice(&[
            0x6f, 0xab, 0x36, 0xbf, 0xbe, 0x12, 0x8e, 0x2b, 0xa8, 0xc4, 0xa6, 0x33, 0xd9,
            0x09, 0x61, 0xbe, 0x75,
        ]);
        let file = NamedTempFile::new().unwrap();
        fs::write(file.path(), file_bytes).unwrap();
        let reader = PstByteReader::open(file.path()).unwrap();
        let bbt = index_with_entry(BlockId(0x12345678), 520, 17);

        let payload =
            load_payload_block(&reader, &bbt, BlockId(0x12345678), ParserLimits::default())
                .unwrap();

        assert_eq!(payload.bytes, b"strong-crypt-test");
        assert_eq!(payload.status, "payload_loaded_strong_crypt_decoded");
    }

    #[test]
    fn rejects_unknown_external_crypt_method_explicitly() {
        let mut file_bytes = vec![0u8; 600];
        file_bytes[PST_HEADER_CRYPT_METHOD_OFFSET] = 7;
        file_bytes[520..524].copy_from_slice(b"data");
        let file = NamedTempFile::new().unwrap();
        fs::write(file.path(), file_bytes).unwrap();
        let reader = PstByteReader::open(file.path()).unwrap();
        let bbt = index_with_entry(BlockId(0x7c), 520, 4);

        let error = load_payload_block(&reader, &bbt, BlockId(0x7c), ParserLimits::default())
            .unwrap_err();
        assert!(error.to_string().contains("unsupported PST data block crypt method"));
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
            status: "test".to_string(),
        }
    }
}
