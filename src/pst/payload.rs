use crate::error::{PstdError, PstdResult};
use crate::pst::bbt::BbtIndex;
use crate::pst::block::{load_block, LoadedBlock};
use crate::pst::limits::ParserLimits;
use crate::pst::header::PST_HEADER_CRYPT_METHOD_OFFSET;
use crate::pst::primitives::{BlockId, BlockRef};
use crate::pst::reader::PstByteReader;

#[derive(Debug, Clone)]
pub struct PayloadBlock {
    pub block_id: BlockId,
    pub block_ref: BlockRef,
    pub bytes: Vec<u8>,
    pub status: String,
}

const NDB_CRYPT_NONE: u8 = 0x00;
const NDB_CRYPT_PERMUTE: u8 = 0x01;
const BID_INTERNAL_MASK: u64 = 0x02;

const PERMUTE_ENCODE_TABLE: [u8; 256] = [
    65, 54, 19, 98, 168, 33, 110, 187, 244, 22, 204, 4, 127, 100, 232, 93, 30, 242, 203,
    42, 116, 197, 94, 53, 210, 149, 71, 158, 150, 45, 154, 136, 76, 125, 132, 63, 219,
    172, 49, 182, 72, 95, 246, 196, 216, 57, 139, 231, 35, 59, 56, 142, 200, 193, 223, 37,
    177, 32, 165, 70, 96, 78, 156, 251, 170, 211, 86, 81, 69, 124, 85, 0, 7, 201, 43,
    157, 133, 155, 9, 160, 143, 173, 179, 15, 99, 171, 137, 75, 215, 167, 21, 90, 113,
    102, 66, 191, 38, 74, 107, 152, 250, 234, 119, 83, 178, 112, 5, 44, 253, 89, 58, 134,
    126, 206, 6, 235, 130, 120, 87, 199, 141, 67, 175, 180, 28, 212, 91, 205, 226, 233,
    39, 79, 195, 8, 114, 128, 207, 176, 239, 245, 40, 109, 190, 48, 77, 52, 146, 213,
    14, 60, 34, 50, 229, 228, 249, 159, 194, 209, 10, 129, 18, 225, 238, 145, 131, 118,
    227, 151, 230, 97, 138, 23, 121, 164, 183, 220, 144, 122, 92, 140, 2, 166, 202, 105,
    222, 80, 26, 17, 147, 185, 82, 135, 88, 252, 237, 29, 55, 73, 27, 106, 224, 41, 51,
    153, 189, 108, 217, 148, 243, 64, 84, 111, 240, 198, 115, 184, 214, 62, 101, 24, 68,
    31, 221, 103, 16, 241, 12, 25, 236, 174, 3, 161, 20, 123, 169, 11, 255, 248, 163,
    192, 162, 1, 247, 46, 188, 36, 104, 117, 13, 254, 186, 47, 181, 208, 218, 61,
];

const fn inverse_permutation() -> [u8; 256] {
    let mut inverse = [0u8; 256];
    let mut index = 0usize;
    while index < 256 {
        inverse[PERMUTE_ENCODE_TABLE[index] as usize] = index as u8;
        index += 1;
    }
    inverse
}

const PERMUTE_DECODE_TABLE: [u8; 256] = inverse_permutation();

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
        "payload_loaded_internal"
    } else {
        match crypt_method {
            NDB_CRYPT_NONE => "payload_loaded_unencoded",
            NDB_CRYPT_PERMUTE => {
                decode_permutative(&mut bytes);
                "payload_loaded_permute_decoded"
            }
            unsupported => {
                return Err(PstdError::pst_parse(
                    Some(block_ref.offset.0),
                    format!("unsupported PST data block crypt method 0x{unsupported:02x}"),
                ));
            }
        }
    };
    Ok(PayloadBlock {
        block_id,
        block_ref,
        bytes,
        status: status.to_string(),
    })
}

fn read_crypt_method(reader: &PstByteReader) -> PstdResult<u8> {
    if reader.file_size() <= PST_HEADER_CRYPT_METHOD_OFFSET as u64 {
        return Ok(NDB_CRYPT_NONE);
    }
    Ok(reader.read_at(PST_HEADER_CRYPT_METHOD_OFFSET as u64, 1)?[0])
}

fn decode_permutative(bytes: &mut [u8]) {
    for byte in bytes {
        *byte = PERMUTE_DECODE_TABLE[*byte as usize];
    }
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
        assert_eq!(payload.status, "payload_loaded_internal");
    }

    #[test]
    fn decodes_permutatively_encoded_external_payload() {
        let plaintext = b"decoded payload";
        let encoded = plaintext
            .iter()
            .map(|byte| super::PERMUTE_ENCODE_TABLE[*byte as usize])
            .collect::<Vec<_>>();
        let mut file_bytes = vec![0u8; 600];
        file_bytes[super::PST_HEADER_CRYPT_METHOD_OFFSET] = super::NDB_CRYPT_PERMUTE;
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
