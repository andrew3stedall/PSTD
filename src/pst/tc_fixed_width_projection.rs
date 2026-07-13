use crate::pst::payload::PayloadBlock;
use crate::pst::tc_fixed_width_evidence::{
    select_fixed_width_row_evidence, FixedWidthRowEvidence,
};
use crate::pst::tc_row_payload_candidates::resolve_row_payload_candidates;
use crate::pst::tc_row_resolution_transport::build_transport_from_row_resolution;
use crate::pst::tc_subnode_rows::TcSubnodeRowResolutionReport;
use crate::pst::tcinfo::TcColumnDescriptor;

pub const TC_FIXED_WIDTH_EVIDENCE_VALIDATED: &str = "tc_fixed_width_evidence_validated";
pub const TC_FIXED_WIDTH_EVIDENCE_UNAVAILABLE: &str = "tc_fixed_width_evidence_unavailable";
pub const TC_FIXED_WIDTH_EVIDENCE_CONSTRUCTION_FAILED: &str =
    "tc_fixed_width_evidence_construction_failed";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TcFixedWidthProjectionReport {
    pub candidate_status: String,
    pub transport_status: String,
    pub evidence_status: String,
    pub evidence: Option<FixedWidthRowEvidence>,
    pub failure_reason: Option<String>,
}

pub fn project_fixed_width_row_evidence(
    payloads: &[PayloadBlock],
    rows_nid: u32,
    resolution: &TcSubnodeRowResolutionReport,
    columns: &[TcColumnDescriptor],
    bitmap_masks: &[String],
    fixed_data_end: usize,
) -> TcFixedWidthProjectionReport {
    let candidates = resolve_row_payload_candidates(payloads, rows_nid);
    let candidate_bytes = candidates
        .payloads
        .iter()
        .map(|payload| payload.bytes.as_slice())
        .collect::<Vec<_>>();
    let transport = build_transport_from_row_resolution(resolution, &candidate_bytes);

    let Some(transport_evidence) = transport.evidence else {
        let evidence_status = if transport.failure_reason.is_some() {
            TC_FIXED_WIDTH_EVIDENCE_CONSTRUCTION_FAILED
        } else {
            TC_FIXED_WIDTH_EVIDENCE_UNAVAILABLE
        };
        return TcFixedWidthProjectionReport {
            candidate_status: candidates.status,
            transport_status: transport.status,
            evidence_status: evidence_status.to_string(),
            evidence: None,
            failure_reason: transport.failure_reason,
        };
    };

    match select_fixed_width_row_evidence(
        columns,
        bitmap_masks,
        &transport_evidence.row_payload,
        &transport_evidence.absolute_row_offsets,
        transport_evidence.row_width,
        fixed_data_end,
    ) {
        Ok(evidence) => TcFixedWidthProjectionReport {
            candidate_status: candidates.status,
            transport_status: transport.status,
            evidence_status: TC_FIXED_WIDTH_EVIDENCE_VALIDATED.to_string(),
            evidence: Some(evidence),
            failure_reason: None,
        },
        Err(reason) => TcFixedWidthProjectionReport {
            candidate_status: candidates.status,
            transport_status: transport.status,
            evidence_status: TC_FIXED_WIDTH_EVIDENCE_CONSTRUCTION_FAILED.to_string(),
            evidence: None,
            failure_reason: Some(reason),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        project_fixed_width_row_evidence, TC_FIXED_WIDTH_EVIDENCE_CONSTRUCTION_FAILED,
        TC_FIXED_WIDTH_EVIDENCE_UNAVAILABLE, TC_FIXED_WIDTH_EVIDENCE_VALIDATED,
    };
    use crate::pst::payload::PayloadBlock;
    use crate::pst::primitives::{BlockId, BlockRef, ByteOffset};
    use crate::pst::tc_subnode_rows::resolve_subnode_row_storage;
    use crate::pst::tcinfo::TcColumnDescriptor;

    #[test]
    fn projects_decoded_scalar_values_from_validated_transport() {
        let mut row_data = vec![0u8; 16];
        for (index, value) in [1i32, 2, 3, 4].iter().enumerate() {
            row_data[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        let payloads = vec![slblock(0x82, &[(0x74, 0x7a)]), payload(0x7a, row_data)];
        let resolution = resolve_subnode_row_storage(&payloads, 0x74, &[0, 1, 2, 3], 1, 4, 4);
        let columns = vec![descriptor(0, 0, 4, 0x0003)];
        let masks = vec!["1".to_string(); 4];

        let report = project_fixed_width_row_evidence(
            &payloads,
            0x74,
            &resolution,
            &columns,
            &masks,
            4,
        );

        assert_eq!(report.evidence_status, TC_FIXED_WIDTH_EVIDENCE_VALIDATED);
        let evidence = report.evidence.expect("validated evidence expected");
        assert_eq!(evidence.property_tag, 0x0003);
        assert_eq!(evidence.row_values_hex, vec!["01000000", "02000000", "03000000", "04000000"]);
        assert_eq!(evidence.decoded_values, vec!["1", "2", "3", "4"]);
    }

    #[test]
    fn emits_no_partial_evidence_when_payload_is_unavailable() {
        let resolution = resolve_subnode_row_storage(&[], 0x74, &[0], 1, 4, 4);
        let report = project_fixed_width_row_evidence(
            &[],
            0x74,
            &resolution,
            &[descriptor(0, 0, 4, 0x0003)],
            &["1".to_string()],
            4,
        );

        assert_eq!(report.evidence_status, TC_FIXED_WIDTH_EVIDENCE_UNAVAILABLE);
        assert!(report.evidence.is_none());
        assert!(report.failure_reason.is_none());
    }

    #[test]
    fn fails_closed_when_bitmap_or_descriptor_evidence_is_invalid() {
        let payloads = vec![slblock(0x82, &[(0x74, 0x7a)]), payload(0x7a, vec![0; 4])];
        let resolution = resolve_subnode_row_storage(&payloads, 0x74, &[0], 1, 4, 4);
        let report = project_fixed_width_row_evidence(
            &payloads,
            0x74,
            &resolution,
            &[descriptor(0, 0, 4, 0x0003)],
            &["0".to_string()],
            4,
        );

        assert_eq!(report.evidence_status, TC_FIXED_WIDTH_EVIDENCE_CONSTRUCTION_FAILED);
        assert!(report.evidence.is_none());
        assert!(report.failure_reason.is_some());
    }

    fn descriptor(bitmap_bit: u8, data_offset: u16, data_size: u8, property_type: u16) -> TcColumnDescriptor {
        TcColumnDescriptor {
            property_tag: property_type as u32,
            data_offset,
            data_size,
            bitmap_bit,
        }
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
