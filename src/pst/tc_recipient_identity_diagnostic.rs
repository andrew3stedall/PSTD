use crate::pst::tc_recipient_identity_projection::{
    RecipientIdentityProjectionReport, RECIPIENT_IDENTITY_UNAVAILABLE,
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TcRecipientIdentityDiagnostic {
    pub candidate_status: String,
    pub transport_status: String,
    pub identity_status: String,
    pub property_tag: Option<u32>,
    pub property_name: Option<String>,
    pub reference_values: Vec<u32>,
    pub reference_kinds: Vec<String>,
    pub row_values: Vec<String>,
    pub failure_reason: Option<String>,
}

impl TcRecipientIdentityDiagnostic {
    pub fn status_fragment(&self) -> String {
        let property_tag = self
            .property_tag
            .map_or_else(|| "none".to_string(), |tag| format!("0x{tag:08x}"));
        let property_name = self.property_name.as_deref().unwrap_or("none");
        let references = self
            .reference_values
            .iter()
            .map(|value| format!("0x{value:08x}"))
            .collect::<Vec<_>>()
            .join(":");
        let kinds = self.reference_kinds.join(":");
        let values = self
            .row_values
            .iter()
            .map(|value| value.replace([';', ':', '|'], ","))
            .collect::<Vec<_>>()
            .join(":");
        let failure = self
            .failure_reason
            .as_deref()
            .unwrap_or("none")
            .replace([';', '|'], ",");

        format!(
            "recipient_candidate_status={},recipient_transport_status={},recipient_identity_status={},recipient_property_tag={},recipient_property_name={},recipient_references={},recipient_reference_kinds={},recipient_values={},recipient_failure={}",
            self.candidate_status.replace(';', ","),
            self.transport_status.replace(';', ","),
            self.identity_status.replace(';', ","),
            property_tag,
            property_name.replace(';', ","),
            references,
            kinds,
            values,
            failure,
        )
    }
}

pub fn build_recipient_identity_diagnostic(
    report: RecipientIdentityProjectionReport,
) -> TcRecipientIdentityDiagnostic {
    let (property_tag, property_name, reference_values, reference_kinds, row_values) =
        match (report.references.as_ref(), report.strings.as_ref()) {
            (Some(references), Some(strings)) => (
                Some(references.property_tag),
                Some(references.property_name.clone()),
                references.row_references.iter().map(|item| item.value).collect(),
                references
                    .row_references
                    .iter()
                    .map(|item| item.kind.clone())
                    .collect(),
                strings.row_values.clone(),
            ),
            _ => (None, None, Vec::new(), Vec::new(), Vec::new()),
        };

    TcRecipientIdentityDiagnostic {
        candidate_status: report.candidate_status,
        transport_status: report.transport_status,
        identity_status: report.identity_status,
        property_tag,
        property_name,
        reference_values,
        reference_kinds,
        row_values,
        failure_reason: report.failure_reason,
    }
}

pub fn unavailable_recipient_identity_diagnostic() -> TcRecipientIdentityDiagnostic {
    TcRecipientIdentityDiagnostic {
        candidate_status: "tc_row_payload_candidates_nid_missing".to_string(),
        transport_status: "tc_row_transport_unavailable".to_string(),
        identity_status: RECIPIENT_IDENTITY_UNAVAILABLE.to_string(),
        property_tag: None,
        property_name: None,
        reference_values: Vec::new(),
        reference_kinds: Vec::new(),
        row_values: Vec::new(),
        failure_reason: None,
    }
}

#[cfg(test)]
mod tests {
    use super::build_recipient_identity_diagnostic;
    use crate::pst::tc_recipient_identity_projection::{
        RecipientIdentityProjectionReport, RECIPIENT_IDENTITY_FAILED,
    };

    #[test]
    fn failed_projection_exposes_no_partial_identity_values() {
        let diagnostic = build_recipient_identity_diagnostic(RecipientIdentityProjectionReport {
            candidate_status: "candidate".to_string(),
            transport_status: "transport".to_string(),
            identity_status: RECIPIENT_IDENTITY_FAILED.to_string(),
            references: None,
            strings: None,
            failure_reason: Some("node-resident reference".to_string()),
        });

        assert!(diagnostic.property_tag.is_none());
        assert!(diagnostic.reference_values.is_empty());
        assert!(diagnostic.row_values.is_empty());
        assert!(diagnostic.status_fragment().contains("recipient_values="));
    }
}
