use crate::error::{PstdError, PstdResult};
use crate::pst::primitives::{BlockRef, ByteOffset};
use crate::pst::reader::PstByteReader;

#[derive(Debug, Clone)]
pub struct LoadedBlock {
    pub block_ref: BlockRef,
    pub bytes: Vec<u8>,
}

pub fn load_block(reader: &PstByteReader, block_ref: BlockRef) -> PstdResult<LoadedBlock> {
    let end = block_ref
        .offset
        .0
        .checked_add(block_ref.size)
        .ok_or_else(|| PstdError::pst_read(Some(block_ref.offset.0), "block range overflowed"))?;

    if end > reader.file_size() {
        return Err(PstdError::pst_read(
            Some(block_ref.offset.0),
            format!(
                "block ends at {end}, beyond file size {}",
                reader.file_size()
            ),
        ));
    }

    let bytes = reader.read_at(block_ref.offset.0, block_ref.size as usize)?;
    Ok(LoadedBlock { block_ref, bytes })
}

#[derive(Debug, Clone)]
pub struct BlockPlan {
    pub first_offset: ByteOffset,
    pub expected_size: u64,
    pub status: String,
}

impl BlockPlan {
    pub fn single(block_ref: BlockRef) -> Self {
        Self {
            first_offset: block_ref.offset,
            expected_size: block_ref.size,
            status: "single_block".to_string(),
        }
    }

    pub fn deferred(first_offset: ByteOffset, expected_size: u64) -> Self {
        Self {
            first_offset,
            expected_size,
            status: "deferred".to_string(),
        }
    }
}
