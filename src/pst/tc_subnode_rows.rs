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
    pub inferred_row_width: usize,
    pub fixed_width_rows: bool,
    pub row_references: Vec<u32>,
    pub row_spans: Vec<usize>,
    pub bitmap_rows_analyzed: usize,
    pub bitmap_set_counts: Vec<usize>,
    pub bitmap_unset_counts: Vec<usize>,
    pub bitmap_masks: Vec<String>,
    pub bitmap_status: String,
    pub status: String,
}

pub fn resolve_subnode_row_storage(
    payloads: &[PayloadBlock],
    rows_nid: u32,
    row_references: &[u32],
    column_count: usize,
    bitmap_start: usize,
    bitmap_end: usize,
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
    let (direct_row_width, direct_fixed_width_rows, row_spans) =
        if resolved_payload_count == 1 && row_references_out_of_bounds == 0 {
            analyze_row_layout(row_references, row_data_byte_len)
        } else {
            (0, false, Vec::new())
        };
    let ordinal_row_width = if resolved_payload_count == 1 && !direct_fixed_width_rows {
        infer_ordinal_row_width(row_references, row_data_byte_len)
    } else {
        0
    };
    let ordinal_index_rows = ordinal_row_width > 0;
    let inferred_row_width = if direct_fixed_width_rows {
        direct_row_width
    } else {
        ordinal_row_width
    };
    let fixed_width_rows = direct_fixed_width_rows || ordinal_index_rows;
    let (bitmap_set_counts, bitmap_unset_counts, bitmap_masks, bitmap_status) =
        if resolved_payload_count == 1 {
            analyze_bitmap_evidence(
                &matching_payloads[0].bytes,
                row_references,
                direct_fixed_width_rows,
                ordinal_index_rows,
                inferred_row_width,
                column_count,
                bitmap_start,
                bitmap_end,
            )
        } else {
            (
                Vec::new(),
                Vec::new(),
                Vec::new(),
                "tc_row_bitmap_payload_unavailable".to_string(),
            )
        };
    let bitmap_rows_analyzed = bitmap_masks.len();
    let status = match (matching_entry_count, resolved_payload_count) {
        (0, _) => "tc_subnode_rows_nid_missing".to_string(),
        (1, 0) => "tc_subnode_rows_payload_missing".to_string(),
        (1, 1) if row_references_out_of_bounds > 0 => {
            "tc_subnode_rows_references_out_of_bounds".to_string()
        }
        (1, 1) if direct_fixed_width_rows => {
            format!("tc_subnode_rows_fixed_width_validated_{inferred_row_width}")
        }
        (1, 1) if ordinal_index_rows => {
            format!("tc_subnode_rows_ordinal_index_validated_{inferred_row_width}")
        }
        (1, 1) => "tc_subnode_rows_variable_or_invalid_width".to_string(),
        _ => "tc_subnode_rows_ambiguous".to_string(),
    };

    TcSubnodeRowResolutionReport {
        rows_nid,
        matching_entry_count,
        resolved_payload_count,
        row_data_byte_len,
        row_reference_count: row_references.len(),
        row_references_in_bounds,
        row_references_out_of_bounds,
        inferred_row_width,
        fixed_width_rows,
        row_references: row_references.to_vec(),
        row_spans,
        bitmap_rows_analyzed,
        bitmap_set_counts,
        bitmap_unset_counts,
        bitmap_masks,
        bitmap_status,
        status,
    }
}

#[allow(clippy::too_many_arguments)]
fn analyze_bitmap_evidence(
    row_data: &[u8],
    row_references: &[u32],
    direct_fixed_width_rows: bool,
    ordinal_index_rows: bool,
    row_width: usize,
    column_count: usize,
    bitmap_start: usize,
    bitmap_end: usize,
) -> (Vec<usize>, Vec<usize>, Vec<String>, String) {
    if row_width == 0 || column_count == 0 || bitmap_start > bitmap_end || bitmap_end > row_width {
        return (
            Vec::new(),
            Vec::new(),
            Vec::new(),
            "tc_row_bitmap_layout_invalid".to_string(),
        );
    }
    if (bitmap_end - bitmap_start) * 8 < column_count {
        return (
            Vec::new(),
            Vec::new(),
            Vec::new(),
            "tc_row_bitmap_capacity_insufficient".to_string(),
        );
    }

    let row_offsets = if direct_fixed_width_rows {
        row_references
            .iter()
            .map(|reference| *reference as usize)
            .collect::<Vec<_>>()
    } else if ordinal_index_rows {
        row_references
            .iter()
            .map(|reference| *reference as usize * row_width)
            .collect::<Vec<_>>()
    } else {
        return (
            Vec::new(),
            Vec::new(),
            Vec::new(),
            "tc_row_bitmap_row_mode_unavailable".to_string(),
        );
    };

    let mut set_counts = Vec::with_capacity(row_offsets.len());
    let mut unset_counts = Vec::with_capacity(row_offsets.len());
    let mut masks = Vec::with_capacity(row_offsets.len());
    for row_offset in row_offsets {
        let start = row_offset.saturating_add(bitmap_start);
        let end = row_offset.saturating_add(bitmap_end);
        if end > row_data.len() || start > end {
            return (
                Vec::new(),
                Vec::new(),
                Vec::new(),
                "tc_row_bitmap_bytes_out_of_bounds".to_string(),
            );
        }
        let bitmap = &row_data[start..end];
        let mask = (0..column_count)
            .map(|bit| {
                if bitmap[bit / 8] & (1 << (bit % 8)) != 0 {
                    '1'
                } else {
                    '0'
                }
            })
            .collect::<String>();
        let set = mask.bytes().filter(|bit| *bit == b'1').count();
        set_counts.push(set);
        unset_counts.push(column_count - set);
        masks.push(mask);
    }

    (
        set_counts,
        unset_counts,
        masks,
        "tc_row_bitmap_masks_validated".to_string(),
    )
}

fn analyze_row_layout(
    row_references: &[u32],
    row_data_byte_len: usize,
) -> (usize, bool, Vec<usize>) {
    if row_references.is_empty() {
        return (0, false, Vec::new());
    }

    let mut offsets = row_references
        .iter()
        .map(|reference| *reference as usize)
        .collect::<Vec<_>>();
    if offsets.windows(2).any(|pair| pair[0] >= pair[1]) {
        return (0, false, Vec::new());
    }
    offsets.push(row_data_byte_len);

    let widths = offsets
        .windows(2)
        .map(|pair| pair[1].saturating_sub(pair[0]))
        .collect::<Vec<_>>();
    let inferred_row_width = widths[0];
    let fixed_width_rows = inferred_row_width > 0
        && offsets[0] == 0
        && widths.iter().all(|width| *width == inferred_row_width);
    (inferred_row_width, fixed_width_rows, widths)
}

fn infer_ordinal_row_width(row_references: &[u32], row_data_byte_len: usize) -> usize {
    if row_references.is_empty() || !row_data_byte_len.is_multiple_of(row_references.len()) {
        return 0;
    }
    if !row_references
        .iter()
        .enumerate()
        .all(|(index, reference)| *reference as usize == index)
    {
        return 0;
    }
    row_data_byte_len / row_references.len()
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
    fn resolves_nid_backed_rows_and_validates_fixed_width() {
        let payloads = vec![slblock(0x82, 0x74, 0x7a), payload(0x7a, vec![0; 12])];
        let report = resolve_subnode_row_storage(&payloads, 0x74, &[0, 4, 8], 8, 3, 4);

        assert_eq!(report.matching_entry_count, 1);
        assert_eq!(report.resolved_payload_count, 1);
        assert_eq!(report.row_data_byte_len, 12);
        assert_eq!(report.row_references_in_bounds, 3);
        assert_eq!(report.row_references_out_of_bounds, 0);
        assert_eq!(report.inferred_row_width, 4);
        assert!(report.fixed_width_rows);
        assert_eq!(report.row_references, vec![0, 4, 8]);
        assert_eq!(report.row_spans, vec![4, 4, 4]);
        assert_eq!(report.bitmap_masks, vec!["00000000"; 3]);
        assert_eq!(report.status, "tc_subnode_rows_fixed_width_validated_4");
    }

    #[test]
    fn validates_contiguous_ordinal_row_references_and_exact_masks() {
        let mut row_data = vec![0; 208];
        for row in 0..4 {
            row_data[row * 52 + 50] = 0b0101_0101;
            row_data[row * 52 + 51] = 0b0001_0101;
        }
        let payloads = vec![slblock(0x82, 0x74, 0x7a), payload(0x7a, row_data)];
        let report = resolve_subnode_row_storage(&payloads, 0x74, &[0, 1, 2, 3], 14, 50, 52);

        assert_eq!(report.inferred_row_width, 52);
        assert!(report.fixed_width_rows);
        assert_eq!(report.row_references, vec![0, 1, 2, 3]);
        assert_eq!(report.row_spans, vec![1, 1, 1, 205]);
        assert_eq!(report.status, "tc_subnode_rows_ordinal_index_validated_52");
        assert_eq!(report.bitmap_rows_analyzed, 4);
        assert_eq!(report.bitmap_set_counts, vec![7, 7, 7, 7]);
        assert_eq!(report.bitmap_unset_counts, vec![7, 7, 7, 7]);
        assert_eq!(report.bitmap_masks, vec!["10101010101010"; 4]);
        assert_eq!(report.bitmap_status, "tc_row_bitmap_masks_validated");
    }

    #[test]
    fn rejects_variable_duplicate_and_nonzero_start_layouts() {
        let payloads = vec![slblock(0x82, 0x74, 0x7a), payload(0x7a, vec![0; 12])];

        for references in [&[0, 4, 9][..], &[0, 4, 4][..], &[2, 6, 10][..]] {
            let report = resolve_subnode_row_storage(&payloads, 0x74, references, 8, 3, 4);
            assert!(!report.fixed_width_rows);
            assert_eq!(report.row_references, references);
            assert_eq!(report.status, "tc_subnode_rows_variable_or_invalid_width");
        }
    }

    #[test]
    fn rejects_noncontiguous_or_nondivisible_ordinal_references() {
        let payloads = vec![slblock(0x82, 0x74, 0x7a), payload(0x7a, vec![0; 13])];
        let nondivisible = resolve_subnode_row_storage(&payloads, 0x74, &[0, 1, 2], 8, 3, 4);
        assert!(!nondivisible.fixed_width_rows);

        let payloads = vec![slblock(0x82, 0x74, 0x7a), payload(0x7a, vec![0; 12])];
        let noncontiguous = resolve_subnode_row_storage(&payloads, 0x74, &[0, 1, 3], 8, 3, 4);
        assert!(!noncontiguous.fixed_width_rows);
    }

    #[test]
    fn reports_missing_and_ambiguous_row_targets() {
        let missing = resolve_subnode_row_storage(&[], 0x74, &[0], 8, 3, 4);
        assert_eq!(missing.status, "tc_subnode_rows_nid_missing");

        let payloads = vec![
            slblock(0x82, 0x74, 0x7a),
            slblock(0x84, 0x74, 0x7c),
            payload(0x7a, vec![0; 8]),
            payload(0x7c, vec![0; 8]),
        ];
        let ambiguous = resolve_subnode_row_storage(&payloads, 0x74, &[0], 8, 3, 4);
        assert_eq!(ambiguous.matching_entry_count, 2);
        assert_eq!(ambiguous.status, "tc_subnode_rows_ambiguous");
    }

    #[test]
    fn bounds_checks_resolved_row_references_before_width_inference() {
        let payloads = vec![slblock(0x82, 0x74, 0x7a), payload(0x7a, vec![0; 4])];
        let report = resolve_subnode_row_storage(&payloads, 0x74, &[0, 4], 8, 3, 4);

        assert_eq!(report.row_references_in_bounds, 1);
        assert_eq!(report.row_references_out_of_bounds, 1);
        assert_eq!(report.inferred_row_width, 0);
        assert!(!report.fixed_width_rows);
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
