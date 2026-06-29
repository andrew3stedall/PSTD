use crate::error::{PstdError, PstdResult};
use crate::pst::binary::{slice_at, u16_le_at, u32_le_at};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HeapHeader {
    pub signature: u16,
    pub client_signature: u16,
    pub user_root: u32,
    pub allocation_count: u16,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HeapAllocation {
    pub id: u16,
    pub offset: u16,
    pub size: u16,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HeapOnNode {
    pub header: HeapHeader,
    pub allocations: Vec<HeapAllocation>,
}

impl HeapOnNode {
    pub fn parse(buf: &[u8], base_offset: u64) -> PstdResult<Self> {
        if buf.len() < 12 {
            return Err(PstdError::pst_parse(Some(base_offset), "heap buffer too short"));
        }

        let header = HeapHeader {
            signature: u16_le_at(buf, 0, base_offset)?,
            client_signature: u16_le_at(buf, 2, base_offset)?,
            user_root: u32_le_at(buf, 4, base_offset)?,
            allocation_count: u16_le_at(buf, 8, base_offset)?,
        };

        let mut allocations = Vec::new();
        let mut cursor = 12usize;
        for idx in 0..header.allocation_count as usize {
            if cursor + 4 > buf.len() {
                break;
            }
            let offset = u16_le_at(buf, cursor, base_offset)?;
            let size = u16_le_at(buf, cursor + 2, base_offset)?;
            allocations.push(HeapAllocation { id: idx as u16, offset, size });
            cursor += 4;
        }

        Ok(Self { header, allocations })
    }

    pub fn allocation<'a>(&self, buf: &'a [u8], id: u16, base_offset: u64) -> PstdResult<&'a [u8]> {
        let allocation = self
            .allocations
            .iter()
            .find(|item| item.id == id)
            .ok_or_else(|| PstdError::pst_parse(Some(base_offset), format!("heap allocation {id} not found")))?;
        slice_at(buf, allocation.offset as usize, allocation.size as usize, base_offset)
    }
}
