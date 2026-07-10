use crate::error::PstdResult;
use crate::pst::heap::HeapOnNode;
use crate::pst::tc_bth::{parse_row_index_bth, TcRowIndexReport};
use crate::pst::tcinfo::{HnidKind, TcInfo};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TcHeapResolutionReport {
    pub user_root_hid: u32,
    pub user_root_resolved: bool,
    pub column_count: usize,
    pub property_tags: Vec<u32>,
    pub row_index_hid: u32,
    pub row_index_resolved: bool,
    pub row_index_report: Option<TcRowIndexReport>,
    pub row_index_error: Option<String>,
    pub rows_hnid: u32,
    pub rows_hnid_kind: HnidKind,
    pub rows_hid_resolved: bool,
    pub rows_requires_subnode_resolution: bool,
    pub row_data_byte_len: usize,
    pub row_reference_count: usize,
    pub row_references_in_bounds: usize,
    pub row_references_out_of_bounds: usize,
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

    let row_index_result = parse_row_index_bth(&heap, bytes, tcinfo.row_index_hid, base_offset);
    let (row_index_report, row_index_error) = match row_index_result {
        Ok(report) => (Some(report), None),
        Err(reason) => (None, Some(reason.to_string())),
    };
    let row_index_resolved = row_index_report.is_some();

    let index_resolved = heap
        .allocation_by_hid(bytes, tcinfo.index_hid, base_offset)
        .is_ok();
    let row_data = if matches!(tcinfo.rows_hnid_kind, HnidKind::HeapId) {
        heap.allocation_by_hid(bytes, tcinfo.rows_hnid, base_offset)
            .ok()
    } else {
        None
    };
    let rows_hid_resolved = row_data.is_some();
    let rows_requires_subnode_resolution = matches!(tcinfo.rows_hnid_kind, HnidKind::NodeId);
    let row_data_byte_len = row_data.map_or(0, <[u8]>::len);
    let row_reference_count = row_index_report
        .as_ref()
        .map_or(0, |report| report.entries.len());
    let row_references_in_bounds = row_index_report.as_ref().map_or(0, |report| {
        report
            .entries
            .iter()
            .filter(|entry| (entry.row_reference as usize) < row_data_byte_len)
            .count()
    });
    let row_references_out_of_bounds = if row_data.is_some() {
        row_reference_count.saturating_sub(row_references_in_bounds)
    } else {
        0
    };

    let status = if !row_index_resolved {
        "tc_heap_row_index_unresolved"
    } else if !index_resolved {
        "tc_heap_index_unresolved"
    } else if rows_requires_subnode_resolution {
        "tc_heap_rows_require_subnode_resolution"
    } else if matches!(tcinfo.rows_hnid_kind, HnidKind::HeapId) && !rows_hid_resolved {
        "tc_heap_rows_hid_unresolved"
    } else if row_references_out_of_bounds > 0 {
        "tc_heap_row_references_out_of_bounds"
    } else {
        "tc_heap_row_references_validated"
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
        row_index_report,
        row_index_error,
        rows_hnid: tcinfo.rows_hnid,
        rows_hnid_kind: tcinfo.rows_hnid_kind,
        rows_hid_resolved,
        rows_requires_subnode_resolution,
        row_data_byte_len,
        row_reference_count,
        row_references_in_bounds,
        row_references_out_of_bounds,
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
    fn resolves_and_validates_heap_backed_row_references() {
        let bytes = sample_tc_heap(0xa0, &[0, 3]);
        let report = resolve_tcinfo_from_heap(&bytes, 0).unwrap();

        assert_eq!(report.user_root_hid, 0x40);
        assert!(report.user_root_resolved);
        assert_eq!(report.column_count, 2);
        assert_eq!(report.property_tags, vec![0x001a0037, 0x001f3001]);
        assert_eq!(report.row_index_hid, 0x60);
        assert!(report.row_index_resolved);
        assert_eq!(report.row_reference_count, 2);
        assert_eq!(report.row_references_in_bounds, 2);
        assert_eq!(report.row_references_out_of_bounds, 0);
        assert_eq!(report.rows_hnid_kind, HnidKind::HeapId);
        assert!(report.rows_hid_resolved);
        assert!(report.index_resolved);
        assert_eq!(report.status, "tc_heap_row_references_validated");
    }

    #[test]
    fn reports_out_of_bounds_heap_row_references() {
        let bytes = sample_tc_heap(0xa0, &[0, 99]);
        let report = resolve_tcinfo_from_heap(&bytes, 0).unwrap();

        assert_eq!(report.row_reference_count, 2);
        assert_eq!(report.row_references_in_bounds, 1);
        assert_eq!(report.row_references_out_of_bounds, 1);
        assert_eq!(report.status, "tc_heap_row_references_out_of_bounds");
    }

    #[test]
    fn preserves_node_id_rows_for_subnode_resolution() {
        let bytes = sample_tc_heap(0x74, &[0, 3]);
        let report = resolve_tcinfo_from_heap(&bytes, 0).unwrap();

        assert_eq!(report.rows_hnid, 0x74);
        assert_eq!(report.rows_hnid_kind, HnidKind::NodeId);
        assert!(!report.rows_hid_resolved);
        assert!(report.rows_requires_subnode_resolution);
        assert_eq!(report.row_reference_count, 2);
        assert_eq!(report.row_references_in_bounds, 0);
        assert_eq!(report.row_references_out_of_bounds, 0);
        assert_eq!(report.status, "tc_heap_rows_require_subnode_resolution");
    }

    fn sample_tc_heap(rows_hnid: u32, row_references: &[u32; 2]) -> Vec<u8> {
        let mut bytes = vec![0u8; 192];
        bytes[0..2].copy_from_slice(&172u16.to_le_bytes());
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
        bytes[root_start + 18..root_start + 22].copy_from_slice(&0xc0u32.to_le_bytes());
        bytes[root_start + 22..root_start + 26].copy_from_slice(&0x001a0037u32.to_le_bytes());
        bytes[root_start + 26..root_start + 28].copy_from_slice(&0u16.to_le_bytes());
        bytes[root_start + 28] = 4;
        bytes[root_start + 29] = 0;
        bytes[root_start + 30..root_start + 34].copy_from_slice(&0x001f3001u32.to_le_bytes());
        bytes[root_start + 34..root_start + 36].copy_from_slice(&4u16.to_le_bytes());
        bytes[root_start + 36] = 4;
        bytes[root_start + 37] = 1;

        bytes[54] = 0xb5;
        bytes[55] = 4;
        bytes[56] = 4;
        bytes[57] = 0;
        bytes[58..62].copy_from_slice(&0x80u32.to_le_bytes());

        bytes[62..66].copy_from_slice(&0x1001u32.to_le_bytes());
        bytes[66..70].copy_from_slice(&row_references[0].to_le_bytes());
        bytes[70..74].copy_from_slice(&0x1002u32.to_le_bytes());
        bytes[74..78].copy_from_slice(&row_references[1].to_le_bytes());

        bytes[78..86].copy_from_slice(b"row-data");
        bytes[86..90].copy_from_slice(b"indx");

        bytes[172..174].copy_from_slice(&6u16.to_le_bytes());
        bytes[174..176].copy_from_slice(&0u16.to_le_bytes());
        for (index, offset) in [8u16, 16, 54, 62, 78, 86, 90].iter().enumerate() {
            let start = 176 + index * 2;
            bytes[start..start + 2].copy_from_slice(&offset.to_le_bytes());
        }
        bytes
    }
}
