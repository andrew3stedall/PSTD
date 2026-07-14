use crate::pst::tc_fixed_width_diagnostic::TcFixedWidthDiagnostic;
use crate::pst::tc_recipient_identity_diagnostic::TcRecipientIdentityDiagnostic;

pub const RECIPIENT_RECORDS_VALIDATED: &str = "tc_recipient_records_validated";
pub const RECIPIENT_RECORDS_UNAVAILABLE: &str = "tc_recipient_records_unavailable";
pub const RECIPIENT_RECORDS_FAILED: &str = "tc_recipient_records_failed";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TcRecipientRecord {
    pub row_index: usize,
    pub role: String,
    pub identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TcRecipientRecordReport {
    pub status: String,
    pub records: Vec<TcRecipientRecord>,
    pub failure_reason: Option<String>,
}

pub fn assemble_recipient_records(
    recipient_types: &TcFixedWidthDiagnostic,
    identities: &TcRecipientIdentityDiagnostic,
) -> TcRecipientRecordReport {
    if recipient_types.property_name.as_deref() != Some("PidTagRecipientType")
        || recipient_types.semantic_values.is_empty()
        || identities.row_values.is_empty()
    {
        return TcRecipientRecordReport {
            status: RECIPIENT_RECORDS_UNAVAILABLE.to_string(),
            records: Vec::new(),
            failure_reason: None,
        };
    }

    if recipient_types.semantic_values.len() != identities.row_values.len() {
        return TcRecipientRecordReport {
            status: RECIPIENT_RECORDS_FAILED.to_string(),
            records: Vec::new(),
            failure_reason: Some(format!(
                "recipient role count {} does not match identity count {}",
                recipient_types.semantic_values.len(),
                identities.row_values.len()
            )),
        };
    }

    let records = recipient_types
        .semantic_values
        .iter()
        .zip(&identities.row_values)
        .enumerate()
        .map(|(row_index, (role, identity))| TcRecipientRecord {
            row_index,
            role: role.clone(),
            identity: identity.clone(),
        })
        .collect();

    TcRecipientRecordReport {
        status: RECIPIENT_RECORDS_VALIDATED.to_string(),
        records,
        failure_reason: None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        assemble_recipient_records, RECIPIENT_RECORDS_FAILED, RECIPIENT_RECORDS_UNAVAILABLE,
        RECIPIENT_RECORDS_VALIDATED,
    };
    use crate::pst::tc_fixed_width_diagnostic::TcFixedWidthDiagnostic;
    use crate::pst::tc_recipient_identity_diagnostic::TcRecipientIdentityDiagnostic;

    fn recipient_types(values: &[&str]) -> TcFixedWidthDiagnostic {
        TcFixedWidthDiagnostic {
            candidate_status: "candidate".to_string(),
            transport_status: "transport".to_string(),
            evidence_status: "validated".to_string(),
            property_tag: Some(0x0c15_0003),
            property_name: Some("PidTagRecipientType".to_string()),
            data_offset: Some(24),
            data_size: Some(4),
            row_values_hex: Vec::new(),
            decoded_values: Vec::new(),
            semantic_values: values.iter().map(|value| (*value).to_string()).collect(),
            failure_reason: None,
        }
    }

    fn identities(values: &[&str]) -> TcRecipientIdentityDiagnostic {
        TcRecipientIdentityDiagnostic {
            candidate_status: "candidate".to_string(),
            transport_status: "transport".to_string(),
            identity_status: "validated".to_string(),
            property_tag: Some(0x3001_001f),
            property_name: Some("PidTagDisplayName".to_string()),
            reference_values: Vec::new(),
            reference_kinds: Vec::new(),
            row_values: values.iter().map(|value| (*value).to_string()).collect(),
            failure_reason: None,
        }
    }

    #[test]
    fn assembles_fixture_recipient_roles_and_names_by_row() {
        let report = assemble_recipient_records(
            &recipient_types(&["to", "to", "cc", "cc"]),
            &identities(&["Recipient 1", "Recipient 2", "Recipient 3", "Recipient 4"]),
        );

        assert_eq!(report.status, RECIPIENT_RECORDS_VALIDATED);
        assert_eq!(report.records.len(), 4);
        assert_eq!(report.records[0].row_index, 0);
        assert_eq!(report.records[0].role, "to");
        assert_eq!(report.records[0].identity, "Recipient 1");
        assert_eq!(report.records[2].role, "cc");
        assert_eq!(report.records[2].identity, "Recipient 3");
        assert!(report.failure_reason.is_none());
    }

    #[test]
    fn mismatched_row_counts_fail_closed_without_partial_records() {
        let report = assemble_recipient_records(
            &recipient_types(&["to", "cc"]),
            &identities(&["Recipient 1"]),
        );

        assert_eq!(report.status, RECIPIENT_RECORDS_FAILED);
        assert!(report.records.is_empty());
        assert!(report.failure_reason.is_some());
    }

    #[test]
    fn missing_role_or_identity_evidence_is_unavailable() {
        let mut types = recipient_types(&[]);
        types.property_name = None;
        let report = assemble_recipient_records(&types, &identities(&["Recipient 1"]));

        assert_eq!(report.status, RECIPIENT_RECORDS_UNAVAILABLE);
        assert!(report.records.is_empty());
        assert!(report.failure_reason.is_none());
    }
}
