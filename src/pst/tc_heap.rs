use crate::error::PstdResult;
use crate::pst::heap::HeapOnNode;
use crate::pst::tcinfo::{HnidKind, TcInfo};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TcHeapResolutionReport {
    pub user_root_hid: u32,
    pub user_root_resolved: bool,
    pub column_count: usize,
    pub property_tags: Vec<u32>,
    pub row_index_hid: u32,
    pub row_index_resolved: bool,
    pub rows_hnid: u32,
    pub rows_hnid_kind: HnidKind,
    pub rows_hid_resolved: bool,
    pub rows_requires_subnode_resolution: bool,
    pub index_hid: u32,
    pub index_resolved: bool,
    pub status: String,
}

pub fn resolve_tcinfo_from_heap(
    bytes: &[u8],
    base_offset: u64,
) -> PstdResult<TcHeapResolutionReport> {
    let heap = HeapOnNode::parse(bytes, base_offset)?;
    let user_root_hid = heap.header.user_root;
    let root = heap.allocation_by_hid(bytes, user_root_hid, base_offset)?;
    let tcinfo = TcInfo::parse(root, base_offset)?;

    let row_index_resolved = heap
        .allocation_by_hid(bytes, tcinfo.row_index_hid, base_offset)
        .is_ok();
    let index_resolved = heap
        .allocation_by_hid(bytes, tcinfo.index_hid, base_offset)
        .is_ok();
    let rows_hid_resolved = matches!(tcinfo.rows_hnid_kind, HnidKind::HeapId)
        && heap
            .allocation_by_hid(bytes, tcinfo.rows_hnid, base_offset)
            .is_ok();
    let rows_requires_subnode_resolution = matches!(tcinfo.rows_hnid_kind, HnidKind::NodeId);

    let status = if !row_index_resolved || !index_resolved {
        "tc_heap_references_partially_resolved"
    } else if rows_requires_subnode_resolution {
        "tc_heap_rows_require_subnode_resolution"
    } else if matches!(tcinfo.rows_hnid_kind, HnidKind::HeapId) && !rows_hid_resolved {
        "tc_heap_rows_hid_unresolved"
    } else {
        "tc_heap_references_resolved"
    };

    Ok(TcHeapResolutionReport {
        user_root_hid,
        user_root_resolved: true,
        column_count: tcinfo.columns.len(),
        property_tags: tcinfo
            .columns
            .iter()
            .map(|column| column.property_tag)
            .collect(),
        row_index_hid: tcinfo.row_index_hid,
        row_index_resolved,
        rows_hnid: tcinfo.rows_hnid,
        rows_hnid_kind: tcinfo.rows_hnid_kind,
        rows_hid_resolved,
        rows_requires_subnode_resolution,
        index_hid: tcinfo.index_hid,
        index_resolved,
        status: status.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::resolve_tcinfo_from_heap;
    use crate::pst::tcinfo::HnidKind;

    #[test]
    fn resolves_real_tcinfo_user_root_and_heap_hids() {
        let bytes = sample_tc_heap(0x80);
        let report = resolve_tcinfo_from_heap(&bytes, 0).unwrap();

        assert_eq!(report.user_root_hid, 0x40);
        assert!(report.user_root_resolved);
        assert_eq!(report.column_count, 2);
        assert_eq!(report.property_tags, vec![0x001a0037, 0x001f3001]);
        assert_eq!(report.row_index_hid, 0x60);
        assert!(report.row_index_resolved);
        assert_eq!(report.rows_hnid_kind, HnidKind::HeapId);
        assert!(report.rows_hid_resolved);
        assert!(report.index_resolved);
        assert_eq!(report.status, "tc_heap_references_resolved");
    }

    #[test]
    fn preserves_node_id_rows_for_subnode_resolution() {
        let bytes = sample_tc_heap(0x74);
        let report = resolve_tcinfo_from_heap(&bytes, 0).unwrap();

        assert_eq!(report.rows_hnid, 0x74);
        assert_eq!(report.rows_hnid_kind, HnidKind::NodeId);
        assert!(!report.rows_hid_resolved);
        assert!(report.rows_requires_subnode_resolution);
        assert_eq!(report.status, "tc_heap_rows_require_subnode_resolution");
    }

    fn sample_tc_heap(rows_hnid: u32) -> Vec<u8> {
        let mut bytes = vec![0u8; 160];
        bytes[0..2].copy_from_slice(&140u16.to_le_bytes());
        bytes[2] = 0xec;
        bytes[3] = 0x7c;
        bytes[4..8].copy_from_slice(&0x40u32.to_le_bytes());

        let root_start = 16usize;
        bytes[root_start] = 0x7c;
        bytes[root_start + 1] = 2;
        bytes[root_start + 2..root_start + 4].copy_from_slice(&4u16.to_le_bytes());
        bytes[root_start + 4..root_start + 6].copy_from_slice(&8u16.to_le_bytes());
        bytes[root_start + 6..root_start + 8].copy_from_slice(&10u16.to_le_bytes());
        bytes[root_start + 8..root_start + 10].copy_from_slice(&12u16.to_le_bytes());
        bytes[root_start + 10..root_start + 14].copy_from_slice(&0x60u32.to_le_bytes());
        bytes[root_start + 14..root_start + 18].copy_from_slice(&rows_hnid.to_le_bytes());
        bytes[root_start + 18..root_start + 22].copy_from_slice(&0xa0u32.to_le_bytes());
        bytes[root_start + 22..root_start + 26].copy_from_slice(&0x001a0037u32.to_le_bytes());
        bytes[root_start + 26..root_start + 28].copy_from_slice(&0u16.to_le_bytes());
        bytes[root_start + 28] = 4;
        bytes[root_start + 29] = 0;
        bytes[root_start + 30..root_start + 34].copy_from_slice(&0x001f3001u32.to_le_bytes());
        bytes[root_start + 34..root_start + 36].copy_from_slice(&4u16.to_le_bytes());
        bytes[root_start + 36] = 4;
        bytes[root_start + 37] = 1;

        bytes[64..68].copy_from_slice(b"ridx");
        bytes[68..72].copy_from_slice(b"rows");
        bytes[72..76].copy_from_slice(b"indx");

        bytes[140..142].copy_from_slice(&5u16.to_le_bytes());
        bytes[142..144].copy_from_slice(&0u16.to_le_bytes());
        for (index, offset) in [8u16, 16, 54, 64, 68, 72].iter().enumerate() {
            let start = 144 + index * 2;
            bytes[start..start + 2].copy_from_slice(&offset.to_le_bytes());
        }
        bytes
    }
}
