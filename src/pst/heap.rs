use crate::error::{PstdError, PstdResult};
use crate::pst::binary::{slice_at, u16_le_at, u32_le_at, u8_at};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HeapHeader {
    pub page_map_offset: u16,
    pub signature: u8,
    pub client_signature: u8,
    pub user_root: u32,
    pub allocation_count: u16,
    pub free_allocation_count: u16,
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
        if buf.len() < 8 {
            return Err(PstdError::pst_parse(
                Some(base_offset),
                "heap buffer too short",
            ));
        }

        let page_map_offset = u16_le_at(buf, 0, base_offset)?;
        let page_map_start = page_map_offset as usize;
        if page_map_start + 4 > buf.len() {
            return Err(PstdError::pst_parse(
                Some(base_offset + page_map_offset as u64),
                "heap page map outside payload",
            ));
        }

        let allocation_count = u16_le_at(buf, page_map_start, base_offset)?;
        let free_allocation_count = u16_le_at(buf, page_map_start + 2, base_offset)?;
        let allocation_offset_count = allocation_count as usize + 1;
        let allocation_offsets_start = page_map_start + 4;
        if allocation_offsets_start + allocation_offset_count * 2 > buf.len() {
            return Err(PstdError::pst_parse(
                Some(base_offset + allocation_offsets_start as u64),
                "heap allocation offset table outside payload",
            ));
        }

        let header = HeapHeader {
            page_map_offset,
            signature: u8_at(buf, 2, base_offset)?,
            client_signature: u8_at(buf, 3, base_offset)?,
            user_root: u32_le_at(buf, 4, base_offset)?,
            allocation_count,
            free_allocation_count,
        };

        let mut allocation_offsets = Vec::with_capacity(allocation_offset_count);
        for idx in 0..allocation_offset_count {
            allocation_offsets.push(u16_le_at(
                buf,
                allocation_offsets_start + idx * 2,
                base_offset,
            )?);
        }

        let mut allocations = Vec::new();
        for idx in 0..allocation_count as usize {
            let offset = allocation_offsets[idx];
            let end = allocation_offsets[idx + 1];
            if end < offset || end as usize > buf.len() {
                continue;
            }
            allocations.push(HeapAllocation {
                id: idx as u16 + 1,
                offset,
                size: end - offset,
            });
        }

        Ok(Self {
            header,
            allocations,
        })
    }

    pub fn allocation<'a>(&self, buf: &'a [u8], id: u16, base_offset: u64) -> PstdResult<&'a [u8]> {
        let allocation = self
            .allocations
            .iter()
            .find(|item| item.id == id)
            .ok_or_else(|| {
                PstdError::pst_parse(Some(base_offset), format!("heap allocation {id} not found"))
            })?;
        slice_at(
            buf,
            allocation.offset as usize,
            allocation.size as usize,
            base_offset,
        )
    }

    pub fn allocation_by_hid<'a>(
        &self,
        buf: &'a [u8],
        hid: u32,
        base_offset: u64,
    ) -> PstdResult<&'a [u8]> {
        let id = hid_index(hid).ok_or_else(|| {
            PstdError::pst_parse(Some(base_offset), format!("invalid heap HID 0x{hid:08x}"))
        })?;
        self.allocation(buf, id, base_offset)
    }

    pub fn try_allocation_by_hnid<'a>(
        &self,
        buf: &'a [u8],
        hnid: u32,
        base_offset: u64,
    ) -> Option<&'a [u8]> {
        self.allocation_by_hid(buf, hnid, base_offset).ok()
    }
}

pub fn hid_index(hid: u32) -> Option<u16> {
    let index = hid >> 5;
    if index == 0 || index > u16::MAX as u32 {
        None
    } else {
        Some(index as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::{hid_index, HeapOnNode};

    #[test]
    fn maps_hid_to_one_based_heap_allocation_index() {
        assert_eq!(hid_index(0x20), Some(1));
        assert_eq!(hid_index(0x40), Some(2));
        assert_eq!(hid_index(0), None);
    }

    #[test]
    fn parses_heap_page_map_and_allocations() {
        let heap_bytes = sample_heap();
        let heap = HeapOnNode::parse(&heap_bytes, 0).unwrap();

        assert_eq!(heap.header.signature, 0xec);
        assert_eq!(heap.header.client_signature, 0xbc);
        assert_eq!(heap.header.user_root, 0x20);
        assert_eq!(heap.header.allocation_count, 2);
        assert_eq!(heap.allocations.len(), 2);
        assert_eq!(heap.allocation(&heap_bytes, 1, 0).unwrap(), b"root");
        assert_eq!(
            heap.allocation_by_hid(&heap_bytes, 0x40, 0).unwrap(),
            b"value"
        );
    }

    fn sample_heap() -> Vec<u8> {
        let mut buf = vec![0; 64];
        buf[0..2].copy_from_slice(&48u16.to_le_bytes());
        buf[2] = 0xec;
        buf[3] = 0xbc;
        buf[4..8].copy_from_slice(&0x20u32.to_le_bytes());
        buf[16..20].copy_from_slice(b"root");
        buf[20..25].copy_from_slice(b"value");
        buf[48..50].copy_from_slice(&2u16.to_le_bytes());
        buf[50..52].copy_from_slice(&0u16.to_le_bytes());
        buf[52..54].copy_from_slice(&16u16.to_le_bytes());
        buf[54..56].copy_from_slice(&20u16.to_le_bytes());
        buf[56..58].copy_from_slice(&25u16.to_le_bytes());
        buf
    }
}
