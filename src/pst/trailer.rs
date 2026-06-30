use crate::error::{PstdError, PstdResult};
use crate::pst::binary::{u16_le_at, u32_le_at, u64_le_at, u8_at};
use crate::pst::primitives::{BlockId, ByteOffset};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PageTrailer {
    pub page_type: u8,
    pub page_level: u8,
    pub signature: u16,
    pub crc: u32,
    pub bid: u64,
    pub offset: u64,
}

impl PageTrailer {
    pub const LEN: usize = 16;

    pub fn parse_from_page(page: &[u8], page_offset: u64) -> PstdResult<Self> {
        if page.len() < Self::LEN {
            return Err(PstdError::pst_parse(
                Some(page_offset),
                "page too short for trailer",
            ));
        }
        let start = page.len() - Self::LEN;
        Self::parse(&page[start..], page_offset + start as u64)
    }

    pub fn parse(buf: &[u8], offset: u64) -> PstdResult<Self> {
        if buf.len() < Self::LEN {
            return Err(PstdError::pst_parse(Some(offset), "page trailer too short"));
        }
        Ok(Self {
            page_type: u8_at(buf, 0, offset)?,
            page_level: u8_at(buf, 1, offset)?,
            signature: u16_le_at(buf, 2, offset)?,
            crc: u32_le_at(buf, 4, offset)?,
            bid: u64_le_at(buf, 8, offset)?,
            offset,
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlockTrailer {
    pub block_id: BlockId,
    pub byte_count: u16,
    pub signature: u16,
    pub crc: u32,
    pub offset: ByteOffset,
}

impl BlockTrailer {
    pub const LEN: usize = 16;

    pub fn parse_from_block(block: &[u8], block_offset: u64) -> PstdResult<Self> {
        if block.len() < Self::LEN {
            return Err(PstdError::pst_parse(
                Some(block_offset),
                "block too short for trailer",
            ));
        }
        let start = block.len() - Self::LEN;
        Self::parse(&block[start..], block_offset + start as u64)
    }

    pub fn parse(buf: &[u8], offset: u64) -> PstdResult<Self> {
        if buf.len() < Self::LEN {
            return Err(PstdError::pst_parse(
                Some(offset),
                "block trailer too short",
            ));
        }
        let byte_count = u16_le_at(buf, 0, offset)?;
        let signature = u16_le_at(buf, 2, offset)?;
        let crc = u32_le_at(buf, 4, offset)?;
        let block_id = BlockId(u64_le_at(buf, 8, offset)?);
        Ok(Self {
            block_id,
            byte_count,
            signature,
            crc,
            offset: ByteOffset(offset),
        })
    }
}
