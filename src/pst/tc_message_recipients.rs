use crate::output::ids;
use crate::output::metadata::RecipientRecord;
use crate::pst::tc_recipient_records::{
    TcCompleteRecipientRecordReport, RECIPIENT_RECORDS_VALIDATED,
};

pub const MESSAGE_RECIPIENTS_ATTACHED: &str = "tc_message_recipients_attached";
pub const MESSAGE_RECIPIENTS_UNAVAILABLE: &str = "tc_message_recipients_unavailable";
pub const MESSAGE_RECIPIENTS_FAILED: &str = "tc_message_recipients_failed";

#[derive(Debug, Clone)]
pub struct TcMessageRecipientReport {
    pub status: String,
    pub recipients: Vec<RecipientRecord>,
    pub failure_reason: Option<String>,
}

pub fn message_recipients_from_complete_records(
    message_key: &str,
    report: &TcCompleteRecipientRecordReport,
) -> TcMessageRecipientReport {
    if report.status != RECIPIENT_RECORDS_VALIDATED || report.records.is_empty() {
        return TcMessageRecipientReport {
            status: MESSAGE_RECIPIENTS_UNAVAILABLE.to_string(),
            recipients: Vec::new(),
            failure_reason: report.failure_reason.clone(),
        };
    }

    let mut recipients = Vec::with_capacity(report.records.len());
    for (expected_row_index, record) in report.records.iter().enumerate() {
        if record.row_index != expected_row_index {
            return failed(format!(
                "recipient row order is not contiguous: expected {expected_row_index}, observed {}",
                record.row_index
            ));
        }
        if !matches!(record.role.as_str(), "to" | "cc" | "bcc" | "originator") {
            return failed(format!("unsupported recipient role {}", record.role));
        }
        if record.display_name.trim().is_empty() || record.address.trim().is_empty() {
            return failed(format!(
                "recipient row {} contains an empty display name or address",
                record.row_index
            ));
        }

        let (address_type, smtp_address, resolution_status) = match record.address_kind.as_str() {
            "smtp_address" => (
                Some("SMTP".to_string()),
                Some(record.address.clone()),
                "smtp_available".to_string(),
            ),
            "native_email_address" => (
                Some("native_email_address".to_string()),
                None,
                "raw_address_preserved".to_string(),
            ),
            other => return failed(format!("unsupported recipient address kind {other}")),
        };

        recipients.push(RecipientRecord {
            message_key: message_key.to_string(),
            recipient_key: ids::recipient_key(message_key, &record.role, record.row_index),
            recipient_type: record.role.clone(),
            display_name: Some(record.display_name.clone()),
            raw_address: Some(record.address.clone()),
            address_type,
            smtp_address,
            resolution_status,
            ordinal: record.row_index as u64,
        });
    }

    TcMessageRecipientReport {
        status: MESSAGE_RECIPIENTS_ATTACHED.to_string(),
        recipients,
        failure_reason: None,
    }
}

fn failed(reason: String) -> TcMessageRecipientReport {
    TcMessageRecipientReport {
        status: MESSAGE_RECIPIENTS_FAILED.to_string(),
        recipients: Vec::new(),
        failure_reason: Some(reason),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        message_recipients_from_complete_records, MESSAGE_RECIPIENTS_ATTACHED,
        MESSAGE_RECIPIENTS_FAILED, MESSAGE_RECIPIENTS_UNAVAILABLE,
    };
    use crate::pst::tc_recipient_records::{
        TcCompleteRecipientRecord, TcCompleteRecipientRecordReport, RECIPIENT_RECORDS_FAILED,
        RECIPIENT_RECORDS_VALIDATED,
    };

    #[test]
    fn converts_the_four_fixture_recipients_without_relabelling_native_addresses_as_smtp() {
        let report = validated(vec![
            record(
                0,
                "to",
                "Recipient 1",
                "to1@domain.com",
                "native_email_address",
            ),
            record(
                1,
                "to",
                "Recipient 2",
                "to2@domain.com",
                "native_email_address",
            ),
            record(
                2,
                "cc",
                "Recipient 3",
                "cc1@domain.com",
                "native_email_address",
            ),
            record(
                3,
                "cc",
                "Recipient 4",
                "cc2@domain.com",
                "native_email_address",
            ),
        ]);

        let converted = message_recipients_from_complete_records("msg_123", &report);

        assert_eq!(converted.status, MESSAGE_RECIPIENTS_ATTACHED);
        assert_eq!(converted.recipients.len(), 4);
        assert_eq!(converted.recipients[0].recipient_type, "to");
        assert_eq!(converted.recipients[2].recipient_type, "cc");
        assert_eq!(converted.recipients[3].ordinal, 3);
        assert_eq!(
            converted.recipients[0].raw_address.as_deref(),
            Some("to1@domain.com")
        );
        assert_eq!(converted.recipients[0].smtp_address, None);
        assert_eq!(
            converted.recipients[0].resolution_status,
            "raw_address_preserved"
        );
    }

    #[test]
    fn preserves_authoritative_smtp_addresses() {
        let report = validated(vec![record(
            0,
            "bcc",
            "Recipient 1",
            "recipient@example.com",
            "smtp_address",
        )]);

        let converted = message_recipients_from_complete_records("msg_123", &report);

        assert_eq!(
            converted.recipients[0].smtp_address.as_deref(),
            Some("recipient@example.com")
        );
        assert_eq!(
            converted.recipients[0].address_type.as_deref(),
            Some("SMTP")
        );
        assert_eq!(converted.recipients[0].resolution_status, "smtp_available");
    }

    #[test]
    fn fails_closed_for_non_contiguous_rows() {
        let report = validated(vec![record(
            1,
            "to",
            "Recipient 1",
            "to1@domain.com",
            "native_email_address",
        )]);

        let converted = message_recipients_from_complete_records("msg_123", &report);

        assert_eq!(converted.status, MESSAGE_RECIPIENTS_FAILED);
        assert!(converted.recipients.is_empty());
        assert!(converted.failure_reason.is_some());
    }

    #[test]
    fn unavailable_or_failed_source_evidence_exposes_no_partial_recipients() {
        let report = TcCompleteRecipientRecordReport {
            status: RECIPIENT_RECORDS_FAILED.to_string(),
            records: vec![record(
                0,
                "to",
                "Recipient 1",
                "to1@domain.com",
                "native_email_address",
            )],
            failure_reason: Some("mismatch".to_string()),
        };

        let converted = message_recipients_from_complete_records("msg_123", &report);

        assert_eq!(converted.status, MESSAGE_RECIPIENTS_UNAVAILABLE);
        assert!(converted.recipients.is_empty());
    }

    fn validated(records: Vec<TcCompleteRecipientRecord>) -> TcCompleteRecipientRecordReport {
        TcCompleteRecipientRecordReport {
            status: RECIPIENT_RECORDS_VALIDATED.to_string(),
            records,
            failure_reason: None,
        }
    }

    fn record(
        row_index: usize,
        role: &str,
        display_name: &str,
        address: &str,
        address_kind: &str,
    ) -> TcCompleteRecipientRecord {
        TcCompleteRecipientRecord {
            row_index,
            role: role.to_string(),
            display_name: display_name.to_string(),
            address: address.to_string(),
            address_kind: address_kind.to_string(),
        }
    }
}
