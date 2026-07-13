use crate::pst::payload::PayloadBlock;

const SLBLOCK_TYPE: u8 = 0x02;
const SLBLOCK_LEAF_LEVEL: u8 = 0x00;
const UNICODE_SLBLOCK_HEADER_BYTES: usize = 8;
const UNICODE_SLENTRY_BYTES: usize = 24;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TcRowPayloadCandidateReport<'a> {
    pub matching_entry_count: usize,
    pub payloads: Vec<&'a PayloadBlock>,
    pub status: String,
}

pub fn resolve_row_payload_candidates<'a>(
    payloads: &'a [PayloadBlock],
    rows_nid: u32,
) -> TcRowPayloadCandidateReport<'a> {
    let data_bids = payloads
        .iter()
        .filter(|payload| is_unicode_slblock(&payload.bytes))
        .flat_map(|payload| matching_data_bids(&payload.bytes, rows_nid))
        .collect::<Vec<_>>();

    let matching_entry_count = data_bids.len();
    let resolved_payloads = data_bids
        .iter()
        .flat_map(|bid| {
            payloads
                .iter()
                .filter(move |payload| payload.block_id.0 == *bid)
        })
        .collect::<Vec<_>>();

    let status = match (matching_entry_count, resolved_payloads.len()) {
        (0, _) => "tc_row_payload_candidates_nid_missing",
        (1, 0) => "tc_row_payload_candidates_payload_missing",
        (1, 1) => "tc_row_payload_candidates_resolved",
        _ => "tc_row_payload_candidates_ambiguous",
    };

    TcRowPayloadCandidateReport {
        matching_entry_count,
        payloads: resolved_payloads,
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
    use super::resolve_row_payload_candidates;
    use crate::pst::payload::PayloadBlock;
    use crate::pst::primitives::{BlockId, BlockRef, ByteOffset};

    #[test]
    fn resolves_exactly_one_matching_row_payload() {
        let payloads = vec![slblock(0x82, &[(0x74, 0x7a)]), payload(0x7a, vec![0x5a; 208])];

        let report = resolve_row_payload_candidates(&payloads, 0x74);

        assert_eq!(report.matching_entry_count, 1);
        assert_eq!(report.payloads.len(), 1);
        assert_eq!(report.payloads[0].block_id.0, 0x7a);
        assert_eq!(report.status, "tc_row_payload_candidates_resolved");
    }

    #[test]
    fn reports_missing_payload_without_partial_candidate() {
        let payloads = vec![slblock(0x82, &[(0x74, 0x7a)])];

        let report = resolve_row_payload_candidates(&payloads, 0x74);

        assert_eq!(report.matching_entry_count, 1);
        assert!(report.payloads.is_empty());
        assert_eq!(report.status, "tc_row_payload_candidates_payload_missing");
    }

    #[test]
    fn preserves_ambiguous_candidates_for_fail_closed_bridge_handling() {
        let payloads = vec![
            slblock(0x82, &[(0x74, 0x7a), (0x74, 0x7c)]),
            payload(0x7a, vec![0; 8]),
            payload(0x7c, vec![0; 8]),
        ];

        let report = resolve_row_payload_candidates(&payloads, 0x74);

        assert_eq!(report.matching_entry_count, 2);
        assert_eq!(report.payloads.len(), 2);
        assert_eq!(report.status, "tc_row_payload_candidates_ambiguous");
    }

    #[test]
    fn ignores_truncated_slblock_entries() {
        let mut bytes = slblock(0x82, &[(0x74, 0x7a)]).bytes;
        bytes[2..4].copy_from_slice(&2u16.to_le_bytes());
        let payloads = vec![payload(0x82, bytes), payload(0x7a, vec![0; 8])];

        let report = resolve_row_payload_candidates(&payloads, 0x74);

        assert_eq!(report.matching_entry_count, 1);
        assert_eq!(report.payloads.len(), 1);
        assert_eq!(report.status, "tc_row_payload_candidates_resolved");
    }

    fn slblock(block_id: u64, entries: &[(u64, u64)]) -> PayloadBlock {
        let mut bytes = vec![0u8; 8 + entries.len() * 24];
        bytes[0] = 0x02;
        bytes[1] = 0x00;
        bytes[2..4].copy_from_slice(&(entries.len() as u16).to_le_bytes());
        for (index, (nid, bid_data)) in entries.iter().enumerate() {
            let start = 8 + index * 24;
            bytes[start..start + 8].copy_from_slice(&nid.to_le_bytes());
            bytes[start + 8..start + 16].copy_from_slice(&bid_data.to_le_bytes());
        }
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
