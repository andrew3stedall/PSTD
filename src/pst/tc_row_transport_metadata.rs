use crate::pst::payload::PayloadBlock;
use crate::pst::tc_row_offsets::TcRowAddressMode;
use crate::pst::tc_row_payload_candidates::resolve_row_payload_candidates;
use crate::pst::tc_row_resolution_transport::build_transport_from_row_resolution;
use crate::pst::tc_subnode_rows::TcSubnodeRowResolutionReport;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TcRowTransportMetadata {
    pub candidate_status: String,
    pub transport_status: String,
    pub address_mode: Option<String>,
    pub row_width: Option<usize>,
    pub absolute_row_offsets: Vec<usize>,
    pub failure_reason: Option<String>,
}

pub fn resolve_row_transport_metadata(
    payloads: &[PayloadBlock],
    rows_nid: u32,
    resolution: &TcSubnodeRowResolutionReport,
) -> TcRowTransportMetadata {
    let candidates = resolve_row_payload_candidates(payloads, rows_nid);
    let candidate_bytes = candidates
        .payloads
        .iter()
        .map(|payload| payload.bytes.as_slice())
        .collect::<Vec<_>>();
    let transport = build_transport_from_row_resolution(resolution, &candidate_bytes);

    match transport.evidence {
        Some(evidence) => TcRowTransportMetadata {
            candidate_status: candidates.status,
            transport_status: transport.status,
            address_mode: Some(
                match evidence.address_mode {
                    TcRowAddressMode::DirectOffsets => "direct_offsets",
                    TcRowAddressMode::OrdinalIndices => "ordinal_indices",
                }
                .to_string(),
            ),
            row_width: Some(evidence.row_width),
            absolute_row_offsets: evidence.absolute_row_offsets,
            failure_reason: None,
        },
        None => TcRowTransportMetadata {
            candidate_status: candidates.status,
            transport_status: transport.status,
            address_mode: None,
            row_width: None,
            absolute_row_offsets: Vec::new(),
            failure_reason: transport.failure_reason,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_row_transport_metadata;
    use crate::pst::payload::PayloadBlock;
    use crate::pst::primitives::{BlockId, BlockRef, ByteOffset};
    use crate::pst::tc_subnode_rows::resolve_subnode_row_storage;

    #[test]
    fn projects_validated_ordinal_metadata_without_payload_bytes() {
        let row_data = vec![0x5a; 208];
        let payloads = vec![slblock(0x82, &[(0x74, 0x7a)]), payload(0x7a, row_data)];
        let resolution = resolve_subnode_row_storage(&payloads, 0x74, &[0, 1, 2, 3], 14, 50, 52);

        let metadata = resolve_row_transport_metadata(&payloads, 0x74, &resolution);

        assert_eq!(
            metadata.candidate_status,
            "tc_row_payload_candidates_resolved"
        );
        assert_eq!(metadata.transport_status, "tc_row_transport_validated");
        assert_eq!(metadata.address_mode.as_deref(), Some("ordinal_indices"));
        assert_eq!(metadata.row_width, Some(52));
        assert_eq!(metadata.absolute_row_offsets, vec![0, 52, 104, 156]);
        assert_eq!(metadata.failure_reason, None);
    }

    #[test]
    fn projects_validated_direct_metadata() {
        let row_data = vec![0x3c; 12];
        let payloads = vec![slblock(0x82, &[(0x74, 0x7a)]), payload(0x7a, row_data)];
        let resolution = resolve_subnode_row_storage(&payloads, 0x74, &[0, 4, 8], 8, 3, 4);

        let metadata = resolve_row_transport_metadata(&payloads, 0x74, &resolution);

        assert_eq!(metadata.transport_status, "tc_row_transport_validated");
        assert_eq!(metadata.address_mode.as_deref(), Some("direct_offsets"));
        assert_eq!(metadata.row_width, Some(4));
        assert_eq!(metadata.absolute_row_offsets, vec![0, 4, 8]);
    }

    #[test]
    fn emits_no_partial_metadata_when_payload_is_unavailable() {
        let resolution = resolve_subnode_row_storage(&[], 0x74, &[0], 8, 3, 4);

        let metadata = resolve_row_transport_metadata(&[], 0x74, &resolution);

        assert_eq!(
            metadata.candidate_status,
            "tc_row_payload_candidates_nid_missing"
        );
        assert_eq!(metadata.transport_status, "tc_row_transport_unavailable");
        assert_eq!(metadata.address_mode, None);
        assert_eq!(metadata.row_width, None);
        assert!(metadata.absolute_row_offsets.is_empty());
        assert_eq!(metadata.failure_reason, None);
    }

    #[test]
    fn emits_no_partial_metadata_for_ambiguous_candidates() {
        let row_data = vec![0; 8];
        let payloads = vec![
            slblock(0x82, &[(0x74, 0x7a), (0x74, 0x7c)]),
            payload(0x7a, row_data.clone()),
            payload(0x7c, row_data),
        ];
        let resolution = resolve_subnode_row_storage(&payloads, 0x74, &[0], 8, 3, 4);

        let metadata = resolve_row_transport_metadata(&payloads, 0x74, &resolution);

        assert_eq!(
            metadata.candidate_status,
            "tc_row_payload_candidates_ambiguous"
        );
        assert_eq!(
            metadata.transport_status,
            "tc_row_transport_construction_failed"
        );
        assert_eq!(metadata.address_mode, None);
        assert_eq!(metadata.row_width, None);
        assert!(metadata.absolute_row_offsets.is_empty());
        assert_eq!(
            metadata.failure_reason.as_deref(),
            Some("exactly one resolved row payload is required")
        );
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
