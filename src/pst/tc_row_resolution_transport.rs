use crate::pst::tc_row_mode::select_validated_row_address_mode;
use crate::pst::tc_row_transport::{
    build_validated_row_transport, TcRowTransportEvidence,
};
use crate::pst::tc_subnode_rows::TcSubnodeRowResolutionReport;

pub const TC_ROW_TRANSPORT_VALIDATED: &str = "tc_row_transport_validated";
pub const TC_ROW_TRANSPORT_UNAVAILABLE: &str = "tc_row_transport_unavailable";
pub const TC_ROW_TRANSPORT_CONSTRUCTION_FAILED: &str =
    "tc_row_transport_construction_failed";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TcRowResolutionTransportReport {
    pub status: String,
    pub evidence: Option<TcRowTransportEvidence>,
    pub failure_reason: Option<String>,
}

pub fn build_transport_from_row_resolution(
    resolution: &TcSubnodeRowResolutionReport,
    resolved_payloads: &[&[u8]],
) -> TcRowResolutionTransportReport {
    if resolution.resolved_payload_count == 0 && resolved_payloads.is_empty() {
        return unavailable();
    }

    if resolution.resolved_payload_count != resolved_payloads.len() {
        return failed("resolved payload count does not match supplied payloads");
    }

    if resolved_payloads.len() != 1 {
        return failed("exactly one resolved row payload is required");
    }

    let row_payload = resolved_payloads[0];
    if row_payload.len() != resolution.row_data_byte_len {
        return failed("resolved row payload length does not match resolution evidence");
    }

    let direct_fixed_width_rows = resolution
        .status
        .starts_with("tc_subnode_rows_fixed_width_validated_");
    let ordinal_index_rows = resolution
        .status
        .starts_with("tc_subnode_rows_ordinal_index_validated_");

    let address_mode = match select_validated_row_address_mode(
        &resolution.row_references,
        resolution.inferred_row_width,
        row_payload.len(),
        direct_fixed_width_rows,
        ordinal_index_rows,
    ) {
        Ok(mode) => mode,
        Err(reason) => return failed(reason),
    };

    match build_validated_row_transport(
        row_payload,
        &resolution.row_references,
        resolution.inferred_row_width,
        address_mode,
    ) {
        Ok(evidence) => TcRowResolutionTransportReport {
            status: TC_ROW_TRANSPORT_VALIDATED.to_string(),
            evidence: Some(evidence),
            failure_reason: None,
        },
        Err(reason) => failed(reason),
    }
}

fn unavailable() -> TcRowResolutionTransportReport {
    TcRowResolutionTransportReport {
        status: TC_ROW_TRANSPORT_UNAVAILABLE.to_string(),
        evidence: None,
        failure_reason: None,
    }
}

fn failed(reason: impl Into<String>) -> TcRowResolutionTransportReport {
    TcRowResolutionTransportReport {
        status: TC_ROW_TRANSPORT_CONSTRUCTION_FAILED.to_string(),
        evidence: None,
        failure_reason: Some(reason.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_transport_from_row_resolution, TC_ROW_TRANSPORT_CONSTRUCTION_FAILED,
        TC_ROW_TRANSPORT_UNAVAILABLE, TC_ROW_TRANSPORT_VALIDATED,
    };
    use crate::pst::payload::PayloadBlock;
    use crate::pst::primitives::{BlockId, BlockRef, ByteOffset};
    use crate::pst::tc_row_offsets::TcRowAddressMode;
    use crate::pst::tc_subnode_rows::resolve_subnode_row_storage;

    #[test]
    fn builds_validated_ordinal_transport_from_resolution() {
        let row_data = vec![0x5a; 208];
        let payloads = vec![slblock(0x82, 0x74, 0x7a), payload(0x7a, row_data.clone())];
        let resolution =
            resolve_subnode_row_storage(&payloads, 0x74, &[0, 1, 2, 3], 14, 50, 52);

        let report = build_transport_from_row_resolution(&resolution, &[&row_data]);

        assert_eq!(report.status, TC_ROW_TRANSPORT_VALIDATED);
        assert_eq!(report.failure_reason, None);
        let evidence = report.evidence.expect("validated transport must be retained");
        assert_eq!(evidence.absolute_row_offsets, vec![0, 52, 104, 156]);
        assert_eq!(evidence.address_mode, TcRowAddressMode::OrdinalIndices);
        assert_eq!(evidence.payload, row_data);
    }

    #[test]
    fn builds_validated_direct_transport_from_resolution() {
        let row_data = vec![0x3c; 12];
        let payloads = vec![slblock(0x82, 0x74, 0x7a), payload(0x7a, row_data.clone())];
        let resolution =
            resolve_subnode_row_storage(&payloads, 0x74, &[0, 4, 8], 8, 3, 4);

        let report = build_transport_from_row_resolution(&resolution, &[&row_data]);

        assert_eq!(report.status, TC_ROW_TRANSPORT_VALIDATED);
        let evidence = report.evidence.expect("validated transport must be retained");
        assert_eq!(evidence.absolute_row_offsets, vec![0, 4, 8]);
        assert_eq!(evidence.address_mode, TcRowAddressMode::DirectOffsets);
    }

    #[test]
    fn reports_unavailable_when_no_row_payload_resolves() {
        let resolution = resolve_subnode_row_storage(&[], 0x74, &[0], 8, 3, 4);

        let report = build_transport_from_row_resolution(&resolution, &[]);

        assert_eq!(report.status, TC_ROW_TRANSPORT_UNAVAILABLE);
        assert_eq!(report.evidence, None);
        assert_eq!(report.failure_reason, None);
    }

    #[test]
    fn rejects_ambiguous_payloads_without_partial_evidence() {
        let row_data = vec![0; 8];
        let payloads = vec![
            slblock(0x82, 0x74, 0x7a),
            slblock(0x84, 0x74, 0x7c),
            payload(0x7a, row_data.clone()),
            payload(0x7c, row_data.clone()),
        ];
        let resolution = resolve_subnode_row_storage(&payloads, 0x74, &[0], 8, 3, 4);

        let report =
            build_transport_from_row_resolution(&resolution, &[&row_data, &row_data]);

        assert_eq!(report.status, TC_ROW_TRANSPORT_CONSTRUCTION_FAILED);
        assert_eq!(report.evidence, None);
        assert_eq!(
            report.failure_reason.as_deref(),
            Some("exactly one resolved row payload is required")
        );
    }

    #[test]
    fn rejects_payload_length_mismatch_without_partial_evidence() {
        let row_data = vec![0; 12];
        let payloads = vec![slblock(0x82, 0x74, 0x7a), payload(0x7a, row_data.clone())];
        let resolution =
            resolve_subnode_row_storage(&payloads, 0x74, &[0, 4, 8], 8, 3, 4);
        let truncated = &row_data[..8];

        let report = build_transport_from_row_resolution(&resolution, &[truncated]);

        assert_eq!(report.status, TC_ROW_TRANSPORT_CONSTRUCTION_FAILED);
        assert_eq!(report.evidence, None);
        assert_eq!(
            report.failure_reason.as_deref(),
            Some("resolved row payload length does not match resolution evidence")
        );
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
