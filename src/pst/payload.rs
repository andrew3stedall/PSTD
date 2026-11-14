use crate::error::{PstdError, PstdResult};
use crate::pst::bbt::BbtIndex;
use crate::pst::block::{load_block, LoadedBlock};
use crate::pst::crypto::{decode_in_place, NDB_CRYPT_NONE};
use crate::pst::header::{
    PST_ANSI_HEADER_CRYPT_METHOD_OFFSET, PST_HEADER_CRYPT_METHOD_OFFSET, PST_INDEX_TYPE_ANSI32,
    PST_INDEX_TYPE_ANSI32A, PST_INDEX_TYPE_OFFSET, PST_INDEX_TYPE_UNICODE64,
    PST_INDEX_TYPE_UNICODE64A,
};
use crate::pst::limits::ParserLimits;
use crate::pst::primitives::{BlockId, BlockRef};
use crate::pst::reader::PstByteReader;
use crate::pst::trailer::BlockTrailer;

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
    verify_block_trailer(reader, block_ref, &bytes)?;
    let internal = block_id.0 & BID_INTERNAL_MASK != 0;
    let crypt_method = reader.crypt_method_or_load(read_crypt_method)?;
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

/// Status-safe marker carried by the error for a block whose stored bytes do not
/// match its trailer checksum.
pub const BLOCK_CHECKSUM_MISMATCH: &str = "block_checksum_mismatch";
/// Marker for a block whose trailer records a different data size than the BBT.
pub const BLOCK_TRAILER_SIZE_MISMATCH: &str = "block_trailer_size_mismatch";

const UNICODE_BLOCK_ALIGNMENT: u64 = 64;

/// Check a Unicode-layout block against its trailer before its bytes are used.
///
/// The trailer sits at the end of the 64-byte aligned block. Its data size must
/// equal the BBT size and its CRC must match the stored (still encoded) bytes;
/// otherwise the block is withheld with an error, so altered bytes are never
/// presented as extracted content. ANSI and 4K-page files, and a block whose
/// aligned extent runs past the end of the file, are not checked.
fn verify_block_trailer(
    reader: &PstByteReader,
    block_ref: BlockRef,
    stored: &[u8],
) -> PstdResult<()> {
    if !reader.unicode_block_trailers_or_load(read_unicode_block_layout)? {
        return Ok(());
    }
    let trailer_len = BlockTrailer::LEN as u64;
    let Some(aligned) = block_ref
        .size
        .checked_add(trailer_len)
        .map(|size| size.div_ceil(UNICODE_BLOCK_ALIGNMENT) * UNICODE_BLOCK_ALIGNMENT)
    else {
        return Ok(());
    };
    let Some(trailer_offset) = block_ref
        .offset
        .0
        .checked_add(aligned)
        .filter(|end| *end <= reader.file_size())
        .map(|end| end - trailer_len)
    else {
        return Ok(());
    };
    let trailer = BlockTrailer::parse(
        &reader.read_at(trailer_offset, BlockTrailer::LEN)?,
        trailer_offset,
    )?;
    if u64::from(trailer.byte_count) != block_ref.size {
        return Err(PstdError::pst_parse(
            Some(block_ref.offset.0),
            format!(
                "{BLOCK_TRAILER_SIZE_MISMATCH}: trailer {} bbt {}",
                trailer.byte_count, block_ref.size
            ),
        ));
    }
    if trailer.crc != block_crc32(stored) {
        return Err(PstdError::pst_parse(
            Some(block_ref.offset.0),
            BLOCK_CHECKSUM_MISMATCH,
        ));
    }
    Ok(())
}

fn read_unicode_block_layout(reader: &PstByteReader) -> PstdResult<bool> {
    if reader.file_size() <= PST_INDEX_TYPE_OFFSET as u64 {
        return Ok(false);
    }
    let index_type = reader.read_at(PST_INDEX_TYPE_OFFSET as u64, 1)?[0];
    Ok(matches!(
        index_type,
        PST_INDEX_TYPE_UNICODE64 | PST_INDEX_TYPE_UNICODE64A
    ))
}

/// MS-PST block CRC: reflected CRC-32 polynomial, initial value zero, no final
/// complement.
fn block_crc32(bytes: &[u8]) -> u32 {
    static TABLE: std::sync::OnceLock<[u32; 256]> = std::sync::OnceLock::new();
    let table = TABLE.get_or_init(|| {
        let mut table = [0u32; 256];
        for (index, slot) in table.iter_mut().enumerate() {
            let mut value = index as u32;
            for _ in 0..8 {
                value = if value & 1 == 1 {
                    (value >> 1) ^ 0xedb8_8320
                } else {
                    value >> 1
                };
            }
            *slot = value;
        }
        table
    });
    let mut crc = 0u32;
    for &byte in bytes {
        crc = table[((crc ^ u32::from(byte)) & 0xff) as usize] ^ (crc >> 8);
    }
    crc
}

/// Test support: `data` laid out as a stored Unicode block, padded to the block
/// alignment and ending in a trailer with the matching size and CRC.
#[cfg(test)]
pub(crate) fn unicode_block_for_tests(data: &[u8]) -> Vec<u8> {
    let aligned = (data.len() + BlockTrailer::LEN).div_ceil(64) * 64;
    let mut block = vec![0u8; aligned];
    block[..data.len()].copy_from_slice(data);
    let trailer = aligned - BlockTrailer::LEN;
    block[trailer..trailer + 2].copy_from_slice(&(data.len() as u16).to_le_bytes());
    block[trailer + 4..trailer + 8].copy_from_slice(&block_crc32(data).to_le_bytes());
    block
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
            0x6f, 0xab, 0x36, 0xbf, 0xbe, 0x12, 0x8e, 0x2b, 0xa8, 0xc4, 0xa6, 0x33, 0xd9, 0x09,
            0x61, 0xbe, 0x75,
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

        let error =
            load_payload_block(&reader, &bbt, BlockId(0x7c), ParserLimits::default()).unwrap_err();
        assert!(error
            .to_string()
            .contains("unsupported PST data block crypt method"));
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
            lookup_index: Default::default(),
            page_limit_reached: false,
        }
    }

    fn unicode_file_with_block(data: &[u8], crc: u32, trailer_size: u16) -> (NamedTempFile, u64) {
        let offset = 0x400usize;
        let aligned = (data.len() + 16).div_ceil(64) * 64;
        let mut bytes = vec![0u8; offset + aligned];
        bytes[crate::pst::header::PST_INDEX_TYPE_OFFSET] =
            crate::pst::header::PST_INDEX_TYPE_UNICODE64;
        bytes[offset..offset + data.len()].copy_from_slice(data);
        let trailer = offset + aligned - 16;
        bytes[trailer..trailer + 2].copy_from_slice(&trailer_size.to_le_bytes());
        bytes[trailer + 4..trailer + 8].copy_from_slice(&crc.to_le_bytes());
        let file = NamedTempFile::new().unwrap();
        fs::write(file.path(), bytes).unwrap();
        (file, offset as u64)
    }

    #[test]
    fn verifies_unicode_block_trailer_checksum_and_size() {
        let data = b"attachment bytes under a checksum";
        let crc = super::block_crc32(data);
        assert_eq!(super::block_crc32(b""), 0);
        assert_ne!(crc, 0);

        // Internal BID (bit 1 set) so the bytes are not run through the cipher.
        let bid = BlockId(0x42);
        let (file, offset) = unicode_file_with_block(data, crc, data.len() as u16);
        let reader = PstByteReader::open(file.path()).unwrap();
        let bbt = index_with_entry(bid, offset, data.len() as u64);
        let payload = load_payload_block(&reader, &bbt, bid, ParserLimits::default()).unwrap();
        assert_eq!(payload.bytes, data);

        let (file, offset) = unicode_file_with_block(data, crc ^ 1, data.len() as u16);
        let reader = PstByteReader::open(file.path()).unwrap();
        let bbt = index_with_entry(bid, offset, data.len() as u64);
        let err = load_payload_block(&reader, &bbt, bid, ParserLimits::default()).unwrap_err();
        assert!(err.to_string().contains(super::BLOCK_CHECKSUM_MISMATCH));

        let (file, offset) = unicode_file_with_block(data, crc, data.len() as u16 + 1);
        let reader = PstByteReader::open(file.path()).unwrap();
        let bbt = index_with_entry(bid, offset, data.len() as u64);
        let err = load_payload_block(&reader, &bbt, bid, ParserLimits::default()).unwrap_err();
        assert!(err.to_string().contains(super::BLOCK_TRAILER_SIZE_MISMATCH));
    }
}
