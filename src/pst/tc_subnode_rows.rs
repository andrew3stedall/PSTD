use crate::pst::payload::PayloadBlock;

const SLBLOCK_TYPE: u8 = 0x02;
const SLBLOCK_LEAF_LEVEL: u8 = 0x00;
const UNICODE_SLBLOCK_HEADER_BYTES: usize = 8;
const UNICODE_SLENTRY_BYTES: usize = 24;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TcSubnodeRowResolutionReport {
    pub rows_nid: u32,
    pub matching_entry_count: usize,
    pub resolved_payload_count: usize,
    pub row_data_byte_len: usize,
    pub row_reference_count: usize,
    pub row_references_in_bounds: usize,
    pub row_references_out_of_bounds: usize,
    pub row_width: usize,
    pub fixed_width_rows: bool,
    pub data_regions_in_bounds: bool,
    pub column_extents_in_bounds: bool,
    pub status: String,
}

pub fn resolve_subnode_row_storage(
    payloads: &[PayloadBlock],
    rows_nid: u32,
    row_references: &[u32],
    required_data_region_end: usize,
    required_column_end: usize,
) -> TcSubnodeRowResolutionReport {
    let data_bids = payloads
        .iter()
        .filter(|payload| is_unicode_slblock(&payload.bytes))
        .flat_map(|payload| matching_data_bids(&payload.bytes, rows_nid))
        .collect::<Vec<_>>();
    let matching_entry_count = data_bids.len();
    let matching_payloads = data_bids
        .iter()
        .flat_map(|bid| {
            payloads
                .iter()
                .filter(move |payload| payload.block_id.0 == *bid)
        })
        .collect::<Vec<_>>();
    let resolved_payload_count = matching_payloads.len();
    let row_data_byte_len = if resolved_payload_count == 1 {
        matching_payloads[0].bytes.len()
    } else {
        0
    };
    let row_references_in_bounds = if resolved_payload_count == 1 {
        row_references
            .iter()
            .filter(|reference| (**reference as usize) < row_data_byte_len)
            .count()
    } else {
        0
    };
    let row_references_out_of_bounds = if resolved_payload_count == 1 {
        row_references
            .len()
            .saturating_sub(row_references_in_bounds)
    } else {
        0
    };
    let row_segments = if resolved_payload_count == 1 && row_references_out_of_bounds == 0 {
        row_references
            .windows(2)
            .map(|pair| pair[1].saturating_sub(pair[0]) as usize)
            .chain(
                row_references
                    .last()
                    .map(|last| row_data_byte_len.saturating_sub(*last as usize)),
            )
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let row_width = row_segments.first().copied().unwrap_or(0);
    let fixed_width_rows = !row_segments.is_empty()
        && row_width > 0
        && row_segments.iter().all(|width| *width == row_width)
        && row_references.windows(2).all(|pair| pair[0] < pair[1]);
    let data_regions_in_bounds = fixed_width_rows && required_data_region_end <= row_width;
    let column_extents_in_bounds = fixed_width_rows && required_column_end <= row_width;
    let status = match (matching_entry_count, resolved_payload_count) {
        (0, _) => "tc_subnode_rows_nid_missing",
        (1, 0) => "tc_subnode_rows_payload_missing",
        (1, 1) if row_references_out_of_bounds > 0 => "tc_subnode_rows_references_out_of_bounds",
        (1, 1) if row_references.is_empty() => "tc_subnode_rows_no_references",
        (1, 1) if !fixed_width_rows => "tc_subnode_rows_variable_width",
        (1, 1) if !data_regions_in_bounds || !column_extents_in_bounds => {
            "tc_subnode_rows_layout_out_of_bounds"
        }
        (1, 1) => "tc_subnode_rows_layout_validated",
        _ => "tc_subnode_rows_ambiguous",
    };

    TcSubnodeRowResolutionReport {
        rows_nid,
        matching_entry_count,
        resolved_payload_count,
        row_data_byte_len,
        row_reference_count: row_references.len(),
        row_references_in_bounds,
        row_references_out_of_bounds,
        row_width,
        fixed_width_rows,
        data_regions_in_bounds,
        column_extents_in_bounds,
        status: status.to_string(),
    }
}

fn is_unicode_slblock(bytes: &[u8]) -> bool {
    bytes.len() >= UNICODE_SLBLOCK_HEADER_BYTES
        && bytes[0] == SLBLOCK_TYPE
        && bytes[1] == SLBLOCK_LEAF_LEVEL
        && bytes[4..8] == [0, 0, 0, 0]
}

fn matching_data_bids(bytes: &[u8], rows_nid: u32) -> Vec<u64> {
    let declared_entry_count = u16::from_le_bytes([bytes[2], bytes[3]]) as usize;
    let available_entry_count =
        bytes.len().saturating_sub(UNICODE_SLBLOCK_HEADER_BYTES) / UNICODE_SLENTRY_BYTES;
    let parsed_entry_count = declared_entry_count.min(available_entry_count);

    (0..parsed_entry_count)
        .filter_map(|index| {
            let start = UNICODE_SLBLOCK_HEADER_BYTES + index * UNICODE_SLENTRY_BYTES;
            let nid = read_u64_le(bytes, start);
            let bid_data = read_u64_le(bytes, start + 8);
            (nid == u64::from(rows_nid) && bid_data != 0).then_some(bid_data)
        })
        .collect()
}

fn read_u64_le(bytes: &[u8], offset: usize) -> u64 {
    let mut value = [0u8; 8];
    value.copy_from_slice(&bytes[offset..offset + 8]);
    u64::from_le_bytes(value)
}

#[cfg(test)]
mod tests {
    use super::resolve_subnode_row_storage;
    use crate::pst::payload::PayloadBlock;
    use crate::pst::primitives::{BlockId, BlockRef, ByteOffset};

    #[test]
    fn resolves_nid_backed_rows_and_validates_references() {
        let payloads = vec![slblock(0x82, 0x74, 0x7a), payload(0x7a, vec![0; 12])];
        let report = resolve_subnode_row_storage(&payloads, 0x74, &[0, 6], 6, 6);

        assert_eq!(report.matching_entry_count, 1);
        assert_eq!(report.resolved_payload_count, 1);
        assert_eq!(report.row_data_byte_len, 12);
        assert_eq!(report.row_references_in_bounds, 2);
        assert_eq!(report.row_references_out_of_bounds, 0);
        assert_eq!(report.row_width, 6);
        assert!(report.fixed_width_rows);
        assert!(report.data_regions_in_bounds);
        assert!(report.column_extents_in_bounds);
        assert_eq!(report.status, "tc_subnode_rows_layout_validated");
    }

    #[test]
    fn reports_missing_and_ambiguous_row_targets() {
        let missing = resolve_subnode_row_storage(&[], 0x74, &[0], 1, 1);
        assert_eq!(missing.status, "tc_subnode_rows_nid_missing");

        let payloads = vec![
            slblock(0x82, 0x74, 0x7a),
            slblock(0x84, 0x74, 0x7c),
            payload(0x7a, vec![0; 8]),
            payload(0x7c, vec![0; 8]),
        ];
        let ambiguous = resolve_subnode_row_storage(&payloads, 0x74, &[0], 1, 1);
        assert_eq!(ambiguous.matching_entry_count, 2);
        assert_eq!(ambiguous.status, "tc_subnode_rows_ambiguous");
    }

    #[test]
    fn rejects_variable_width_and_column_overflow() {
        let payloads = vec![slblock(0x82, 0x74, 0x7a), payload(0x7a, vec![0; 12])];
        let variable = resolve_subnode_row_storage(&payloads, 0x74, &[0, 5], 5, 5);
        assert_eq!(variable.status, "tc_subnode_rows_variable_width");

        let overflow = resolve_subnode_row_storage(&payloads, 0x74, &[0, 6], 7, 8);
        assert_eq!(overflow.row_width, 6);
        assert!(!overflow.data_regions_in_bounds);
        assert!(!overflow.column_extents_in_bounds);
        assert_eq!(overflow.status, "tc_subnode_rows_layout_out_of_bounds");
    }

    #[test]
    fn bounds_checks_resolved_row_references() {
        let payloads = vec![slblock(0x82, 0x74, 0x7a), payload(0x7a, vec![0; 4])];
        let report = resolve_subnode_row_storage(&payloads, 0x74, &[0, 4], 1, 1);

        assert_eq!(report.row_references_in_bounds, 1);
        assert_eq!(report.row_references_out_of_bounds, 1);
        assert_eq!(report.status, "tc_subnode_rows_references_out_of_bounds");
    }

    fn slblock(block_id: u64, nid: u64, bid_data: u64) -> PayloadBlock {
        let mut bytes = vec![0u8; 32];
        bytes[0] = 0x02;
        bytes[1] = 0x00;
        bytes[2..4].copy_from_slice(&1u16.to_le_bytes());
        bytes[8..16].copy_from_slice(&nid.to_le_bytes());
        bytes[16..24].copy_from_slice(&bid_data.to_le_bytes());
        payload(block_id, bytes)
    }

    fn payload(block_id: u64, bytes: Vec<u8>) -> PayloadBlock {
        PayloadBlock {
            block_id: BlockId(block_id),
            block_ref: BlockRef {
                block_id: BlockId(block_id),
                offset: ByteOffset(0),
                size: bytes.len() as u64,
            },
            bytes,
            status: "test".to_string(),
        }
    }
}
