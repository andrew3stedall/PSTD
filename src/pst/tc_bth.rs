use std::collections::HashSet;

use crate::error::{PstdError, PstdResult};
use crate::pst::heap::HeapOnNode;

const BTH_HEADER_TYPE: u8 = 0xb5;
const MAX_ROW_INDEX_ENTRIES: usize = 4096;
const MAX_ROW_INDEX_LEVELS: u8 = 8;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TcRowIndexEntry {
    pub row_key: u32,
    pub row_reference: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TcRowIndexReport {
    pub header_hid: u32,
    pub key_size: u8,
    pub value_size: u8,
    pub index_levels: u8,
    pub root_hid: u32,
    pub visited_allocation_count: usize,
    pub entry_count: usize,
    pub truncated: bool,
    pub entries: Vec<TcRowIndexEntry>,
    pub status: String,
}

pub fn parse_row_index_bth(
    heap: &HeapOnNode,
    heap_bytes: &[u8],
    header_hid: u32,
    base_offset: u64,
) -> PstdResult<TcRowIndexReport> {
    let header = heap.allocation_by_hid(heap_bytes, header_hid, base_offset)?;
    if header.len() < 8 {
        return Err(PstdError::pst_parse(
            Some(base_offset),
            "row-index BTH header shorter than 8 bytes",
        ));
    }
    if header[0] != BTH_HEADER_TYPE {
        return Err(PstdError::pst_parse(
            Some(base_offset),
            format!("row-index BTH type mismatch: 0x{:02x}", header[0]),
        ));
    }

    let key_size = header[1];
    let value_size = header[2];
    let index_levels = header[3];
    if key_size != 4 || value_size != 4 {
        return Err(PstdError::pst_parse(
            Some(base_offset),
            format!(
                "unsupported row-index BTH entry sizes: key={key_size}, value={value_size}"
            ),
        ));
    }
    if index_levels > MAX_ROW_INDEX_LEVELS {
        return Err(PstdError::pst_parse(
            Some(base_offset),
            format!("row-index BTH levels exceed limit: {index_levels}"),
        ));
    }

    let root_hid = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
    let mut entries = Vec::new();
    let mut visited = HashSet::new();
    let mut truncated = false;
    traverse_row_index(
        heap,
        heap_bytes,
        root_hid,
        index_levels,
        base_offset,
        &mut visited,
        &mut entries,
        &mut truncated,
    )?;

    let status = if truncated {
        "tc_row_index_bth_truncated"
    } else if entries.is_empty() {
        "tc_row_index_bth_empty"
    } else {
        "tc_row_index_bth_parsed"
    };

    Ok(TcRowIndexReport {
        header_hid,
        key_size,
        value_size,
        index_levels,
        root_hid,
        visited_allocation_count: visited.len(),
        entry_count: entries.len(),
        truncated,
        entries,
        status: status.to_string(),
    })
}

#[allow(clippy::too_many_arguments)]
fn traverse_row_index(
    heap: &HeapOnNode,
    heap_bytes: &[u8],
    hid: u32,
    level: u8,
    base_offset: u64,
    visited: &mut HashSet<u32>,
    entries: &mut Vec<TcRowIndexEntry>,
    truncated: &mut bool,
) -> PstdResult<()> {
    if !visited.insert(hid) {
        return Err(PstdError::pst_parse(
            Some(base_offset),
            format!("row-index BTH cycle detected at HID 0x{hid:08x}"),
        ));
    }
    let allocation = heap.allocation_by_hid(heap_bytes, hid, base_offset)?;

    if level == 0 {
        for chunk in allocation.chunks_exact(8) {
            if entries.len() >= MAX_ROW_INDEX_ENTRIES {
                *truncated = true;
                break;
            }
            entries.push(TcRowIndexEntry {
                row_key: u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]),
                row_reference: u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]),
            });
        }
        return Ok(());
    }

    for chunk in allocation.chunks_exact(8) {
        if entries.len() >= MAX_ROW_INDEX_ENTRIES {
            *truncated = true;
            break;
        }
        let child_hid = u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
        traverse_row_index(
            heap,
            heap_bytes,
            child_hid,
            level - 1,
            base_offset,
            visited,
            entries,
            truncated,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_row_index_bth;
    use crate::pst::heap::HeapOnNode;

    #[test]
    fn parses_leaf_row_index_entries() {
        let bytes = sample_row_index_heap(0);
        let heap = HeapOnNode::parse(&bytes, 0).unwrap();
        let report = parse_row_index_bth(&heap, &bytes, 0x20, 0).unwrap();

        assert_eq!(report.index_levels, 0);
        assert_eq!(report.visited_allocation_count, 1);
        assert_eq!(report.entry_count, 2);
        assert_eq!(report.entries[0].row_key, 0x1001);
        assert_eq!(report.entries[0].row_reference, 0);
        assert_eq!(report.entries[1].row_key, 0x1002);
        assert_eq!(report.entries[1].row_reference, 24);
        assert_eq!(report.status, "tc_row_index_bth_parsed");
    }

    #[test]
    fn traverses_one_index_level() {
        let bytes = sample_row_index_heap(1);
        let heap = HeapOnNode::parse(&bytes, 0).unwrap();
        let report = parse_row_index_bth(&heap, &bytes, 0x20, 0).unwrap();

        assert_eq!(report.index_levels, 1);
        assert_eq!(report.visited_allocation_count, 2);
        assert_eq!(report.entry_count, 2);
    }

    #[test]
    fn rejects_unsupported_row_index_shapes() {
        let mut bytes = sample_row_index_heap(0);
        bytes[9] = 2;
        let heap = HeapOnNode::parse(&bytes, 0).unwrap();
        let error = parse_row_index_bth(&heap, &bytes, 0x20, 0).unwrap_err();
        assert!(error
            .to_string()
            .contains("unsupported row-index BTH entry sizes"));
    }

    fn sample_row_index_heap(index_levels: u8) -> Vec<u8> {
        let mut bytes = vec![0u8; 128];
        bytes[0..2].copy_from_slice(&112u16.to_le_bytes());
        bytes[2] = 0xec;
        bytes[3] = 0x7c;
        bytes[4..8].copy_from_slice(&0x20u32.to_le_bytes());

        bytes[8] = 0xb5;
        bytes[9] = 4;
        bytes[10] = 4;
        bytes[11] = index_levels;
        bytes[12..16].copy_from_slice(&0x40u32.to_le_bytes());

        if index_levels == 0 {
            bytes[16..20].copy_from_slice(&0x1001u32.to_le_bytes());
            bytes[20..24].copy_from_slice(&0u32.to_le_bytes());
            bytes[24..28].copy_from_slice(&0x1002u32.to_le_bytes());
            bytes[28..32].copy_from_slice(&24u32.to_le_bytes());
            bytes[112..114].copy_from_slice(&2u16.to_le_bytes());
            bytes[114..116].copy_from_slice(&0u16.to_le_bytes());
            for (index, offset) in [8u16, 16, 32, 32].iter().enumerate() {
                let start = 116 + index * 2;
                bytes[start..start + 2].copy_from_slice(&offset.to_le_bytes());
            }
        } else {
            bytes[16..20].copy_from_slice(&0x1001u32.to_le_bytes());
            bytes[20..24].copy_from_slice(&0x60u32.to_le_bytes());
            bytes[24..28].copy_from_slice(&0x1001u32.to_le_bytes());
            bytes[28..32].copy_from_slice(&0u32.to_le_bytes());
            bytes[32..36].copy_from_slice(&0x1002u32.to_le_bytes());
            bytes[36..40].copy_from_slice(&24u32.to_le_bytes());
            bytes[112..114].copy_from_slice(&3u16.to_le_bytes());
            bytes[114..116].copy_from_slice(&0u16.to_le_bytes());
            for (index, offset) in [8u16, 16, 24, 40, 40].iter().enumerate() {
                let start = 116 + index * 2;
                bytes[start..start + 2].copy_from_slice(&offset.to_le_bytes());
            }
        }
        bytes
    }
}
