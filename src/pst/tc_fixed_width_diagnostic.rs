use crate::pst::tc_fixed_width_projection::TcFixedWidthProjectionReport;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TcFixedWidthDiagnostic {
    pub candidate_status: String,
    pub transport_status: String,
    pub evidence_status: String,
    pub property_tag: Option<u32>,
    pub data_offset: Option<u16>,
    pub data_size: Option<u8>,
    pub row_values_hex: Vec<String>,
    pub decoded_values: Vec<String>,
    pub failure_reason: Option<String>,
}

impl TcFixedWidthDiagnostic {
    pub fn status_fragment(&self) -> String {
        let property_tag = self
            .property_tag
            .map(|value| format!("0x{value:08x}"))
            .unwrap_or_else(|| "none".to_string());
        let data_offset = self
            .data_offset
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string());
        let data_size = self
            .data_size
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string());
        let row_values_hex = if self.row_values_hex.is_empty() {
            "none".to_string()
        } else {
            self.row_values_hex.join(":")
        };
        let decoded_values = if self.decoded_values.is_empty() {
            "none".to_string()
        } else {
            self.decoded_values.join(":")
        };
        let failure_reason = self
            .failure_reason
            .as_deref()
            .unwrap_or("none")
            .replace(';', ",");

        format!(
            "fixed_candidate_status={},fixed_transport_status={},fixed_evidence_status={},fixed_property_tag={},fixed_data_offset={},fixed_data_size={},fixed_raw_values={},fixed_decoded_values={},fixed_failure_reason={}",
            self.candidate_status.replace(';', ","),
            self.transport_status.replace(';', ","),
            self.evidence_status.replace(';', ","),
            property_tag,
            data_offset,
            data_size,
            row_values_hex,
            decoded_values,
            failure_reason
        )
    }
}

pub fn build_fixed_width_diagnostic(
    projection: TcFixedWidthProjectionReport,
) -> TcFixedWidthDiagnostic {
    let evidence = projection.evidence;
    TcFixedWidthDiagnostic {
        candidate_status: projection.candidate_status,
        transport_status: projection.transport_status,
        evidence_status: projection.evidence_status,
        property_tag: evidence.as_ref().map(|item| item.property_tag),
        data_offset: evidence.as_ref().map(|item| item.data_offset),
        data_size: evidence.as_ref().map(|item| item.data_size),
        row_values_hex: evidence
            .as_ref()
            .map_or_else(Vec::new, |item| item.row_values_hex.clone()),
        decoded_values: evidence
            .as_ref()
            .map_or_else(Vec::new, |item| item.decoded_values.clone()),
        failure_reason: projection.failure_reason,
    }
}

#[cfg(test)]
mod tests {
    use super::build_fixed_width_diagnostic;
    use crate::pst::tc_fixed_width_evidence::FixedWidthRowEvidence;
    use crate::pst::tc_fixed_width_projection::TcFixedWidthProjectionReport;

    #[test]
    fn publishes_validated_values_without_payload_bytes() {
        let diagnostic = build_fixed_width_diagnostic(TcFixedWidthProjectionReport {
            candidate_status: "tc_row_payload_candidates_resolved".to_string(),
            transport_status: "tc_row_transport_validated".to_string(),
            evidence_status: "tc_fixed_width_evidence_validated".to_string(),
            evidence: Some(FixedWidthRowEvidence {
                bitmap_index: 2,
                descriptor_order: 3,
                property_tag: 0x0003,
                data_offset: 8,
                data_size: 4,
                row_values_hex: vec!["01000000".to_string(), "02000000".to_string()],
                decoded_values: vec!["1".to_string(), "2".to_string()],
                distinct_value_count: 2,
            }),
            failure_reason: None,
        });

        assert_eq!(diagnostic.property_tag, Some(0x0003));
        assert_eq!(diagnostic.row_values_hex, ["01000000", "02000000"]);
        assert_eq!(diagnostic.decoded_values, ["1", "2"]);
        let fragment = diagnostic.status_fragment();
        assert!(fragment.contains("fixed_property_tag=0x00000003"));
        assert!(fragment.contains("fixed_decoded_values=1:2"));
    }

    #[test]
    fn suppresses_partial_metadata_when_projection_has_no_evidence() {
        let diagnostic = build_fixed_width_diagnostic(TcFixedWidthProjectionReport {
            candidate_status: "tc_row_payload_candidates_ambiguous".to_string(),
            transport_status: "tc_row_transport_construction_failed".to_string(),
            evidence_status: "tc_fixed_width_evidence_construction_failed".to_string(),
            evidence: None,
            failure_reason: Some("multiple payload candidates".to_string()),
        });

        assert_eq!(diagnostic.property_tag, None);
        assert_eq!(diagnostic.data_offset, None);
        assert_eq!(diagnostic.data_size, None);
        assert!(diagnostic.row_values_hex.is_empty());
        assert!(diagnostic.decoded_values.is_empty());
        let fragment = diagnostic.status_fragment();
        assert!(fragment.contains("fixed_property_tag=none"));
        assert!(fragment.contains("fixed_raw_values=none"));
        assert!(fragment.contains("fixed_decoded_values=none"));
    }
}
