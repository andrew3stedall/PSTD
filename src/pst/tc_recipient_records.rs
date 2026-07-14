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
pub struct TcCompleteRecipientRecord {
    pub row_index: usize,
    pub role: String,
    pub display_name: String,
    pub address: String,
    pub address_kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TcRecipientRecordReport {
    pub status: String,
    pub records: Vec<TcRecipientRecord>,
    pub failure_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TcCompleteRecipientRecordReport {
    pub status: String,
    pub records: Vec<TcCompleteRecipientRecord>,
    pub failure_reason: Option<String>,
}

impl TcRecipientRecordReport {
    pub fn status_fragment(&self) -> String {
        let records = if self.records.is_empty() {
            "none".to_string()
        } else {
            self.records
                .iter()
                .map(|record| {
                    format!(
                        "{}:{}:{}",
                        record.row_index,
                        sanitize(&record.role),
                        sanitize(&record.identity)
                    )
                })
                .collect::<Vec<_>>()
                .join("|")
        };
        let failure = self
            .failure_reason
            .as_deref()
            .map(sanitize)
            .unwrap_or_else(|| "none".to_string());
        format!(
            "recipient_records_status={};recipient_records={};recipient_records_failure={}",
            sanitize(&self.status),
            records,
            failure
        )
    }
}

impl TcCompleteRecipientRecordReport {
    pub fn status_fragment(&self) -> String {
        let records = if self.records.is_empty() {
            "none".to_string()
        } else {
            self.records
                .iter()
                .map(|record| {
                    format!(
                        "{}:{}:{}:{}:{}",
                        record.row_index,
                        sanitize(&record.role),
                        sanitize(&record.display_name),
                        sanitize(&record.address),
                        sanitize(&record.address_kind)
                    )
                })
                .collect::<Vec<_>>()
                .join("|")
        };
        let failure = self
            .failure_reason
            .as_deref()
            .map(sanitize)
            .unwrap_or_else(|| "none".to_string());
        format!(
            "complete_recipient_records_status={};complete_recipient_records={};complete_recipient_records_failure={}",
            sanitize(&self.status),
            records,
            failure
        )
    }
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

pub fn assemble_complete_recipient_records(
    recipient_types: &TcFixedWidthDiagnostic,
    display_names: &TcRecipientIdentityDiagnostic,
    addresses: &TcRecipientIdentityDiagnostic,
) -> TcCompleteRecipientRecordReport {
    if recipient_types.property_name.as_deref() != Some("PidTagRecipientType")
        || display_names.property_name.as_deref() != Some("PidTagDisplayName")
        || !matches!(
            addresses.property_name.as_deref(),
            Some("PidTagSmtpAddress" | "PidTagEmailAddress")
        )
        || recipient_types.semantic_values.is_empty()
        || display_names.row_values.is_empty()
        || addresses.row_values.is_empty()
    {
        return TcCompleteRecipientRecordReport {
            status: RECIPIENT_RECORDS_UNAVAILABLE.to_string(),
            records: Vec::new(),
            failure_reason: None,
        };
    }

    let role_count = recipient_types.semantic_values.len();
    let name_count = display_names.row_values.len();
    let address_count = addresses.row_values.len();
    if role_count != name_count || role_count != address_count {
        return TcCompleteRecipientRecordReport {
            status: RECIPIENT_RECORDS_FAILED.to_string(),
            records: Vec::new(),
            failure_reason: Some(format!(
                "recipient row counts differ: roles {role_count}, display names {name_count}, addresses {address_count}"
            )),
        };
    }

    let address_kind = match addresses.property_name.as_deref() {
        Some("PidTagSmtpAddress") => "smtp_address",
        Some("PidTagEmailAddress") => "native_email_address",
        _ => unreachable!("address property was validated above"),
    };
    let records = recipient_types
        .semantic_values
        .iter()
        .zip(&display_names.row_values)
        .zip(&addresses.row_values)
        .enumerate()
        .map(
            |(row_index, ((role, display_name), address))| TcCompleteRecipientRecord {
                row_index,
                role: role.clone(),
                display_name: display_name.clone(),
                address: address.clone(),
                address_kind: address_kind.to_string(),
            },
        )
        .collect();

    TcCompleteRecipientRecordReport {
        status: RECIPIENT_RECORDS_VALIDATED.to_string(),
        records,
        failure_reason: None,
    }
}

fn sanitize(value: &str) -> String {
    value.replace(';', ",").replace('|', "/").replace(':', "-")
}

#[cfg(test)]
mod tests {
    use super::{
        assemble_complete_recipient_records, assemble_recipient_records, RECIPIENT_RECORDS_FAILED,
        RECIPIENT_RECORDS_UNAVAILABLE, RECIPIENT_RECORDS_VALIDATED,
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

    fn identities(property_name: &str, values: &[&str]) -> TcRecipientIdentityDiagnostic {
        TcRecipientIdentityDiagnostic {
            candidate_status: "candidate".to_string(),
            transport_status: "transport".to_string(),
            identity_status: "validated".to_string(),
            property_tag: Some(match property_name {
                "PidTagDisplayName" => 0x3001_001f,
                "PidTagEmailAddress" => 0x3003_001f,
                "PidTagSmtpAddress" => 0x39fe_001f,
                _ => 0,
            }),
            property_name: Some(property_name.to_string()),
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
            &identities(
                "PidTagDisplayName",
                &["Recipient 1", "Recipient 2", "Recipient 3", "Recipient 4"],
            ),
        );
        assert_eq!(report.status, RECIPIENT_RECORDS_VALIDATED);
        assert_eq!(report.records.len(), 4);
        assert_eq!(report.records[0].role, "to");
        assert_eq!(report.records[2].identity, "Recipient 3");
    }

    #[test]
    fn assembles_complete_fixture_recipient_records_by_row() {
        let report = assemble_complete_recipient_records(
            &recipient_types(&["to", "to", "cc", "cc"]),
            &identities(
                "PidTagDisplayName",
                &["Recipient 1", "Recipient 2", "Recipient 3", "Recipient 4"],
            ),
            &identities(
                "PidTagEmailAddress",
                &[
                    "to1@domain.com",
                    "to2@domain.com",
                    "cc1@domain.com",
                    "cc2@domain.com",
                ],
            ),
        );
        assert_eq!(report.status, RECIPIENT_RECORDS_VALIDATED);
        assert_eq!(report.records.len(), 4);
        assert_eq!(report.records[0].display_name, "Recipient 1");
        assert_eq!(report.records[0].address, "to1@domain.com");
        assert_eq!(report.records[0].address_kind, "native_email_address");
        assert_eq!(report.records[2].role, "cc");
        assert!(report.failure_reason.is_none());
        assert!(report
            .status_fragment()
            .contains("0:to:Recipient 1:to1@domain.com:native_email_address"));
    }

    #[test]
    fn complete_records_fail_closed_when_row_counts_differ() {
        let report = assemble_complete_recipient_records(
            &recipient_types(&["to", "cc"]),
            &identities("PidTagDisplayName", &["Recipient 1"]),
            &identities("PidTagEmailAddress", &["to1@domain.com", "cc1@domain.com"]),
        );
        assert_eq!(report.status, RECIPIENT_RECORDS_FAILED);
        assert!(report.records.is_empty());
        assert!(report.failure_reason.is_some());
    }

    #[test]
    fn complete_records_require_authoritative_name_and_address_properties() {
        let report = assemble_complete_recipient_records(
            &recipient_types(&["to"]),
            &identities("PidTagEmailAddress", &["not a display name"]),
            &identities("PidTagDisplayName", &["not an address"]),
        );
        assert_eq!(report.status, RECIPIENT_RECORDS_UNAVAILABLE);
        assert!(report.records.is_empty());
    }

    #[test]
    fn mismatched_legacy_row_counts_fail_closed_without_partial_records() {
        let report = assemble_recipient_records(
            &recipient_types(&["to", "cc"]),
            &identities("PidTagDisplayName", &["Recipient 1"]),
        );
        assert_eq!(report.status, RECIPIENT_RECORDS_FAILED);
        assert!(report.records.is_empty());
    }
}
