use crate::output::metadata::RecipientRecord;
use crate::pst::tc_message_recipient_selection::{
    select_message_recipient_projection, MESSAGE_RECIPIENT_PROJECTION_SELECTED,
};
use crate::pst::tc_message_recipients::{
    message_recipients_from_complete_records, MESSAGE_RECIPIENTS_ATTACHED,
};
use crate::pst::tc_reporting::TcSubnodeProbeReport;

pub const MESSAGE_RECIPIENT_OUTPUT_ATTACHED: &str = "tc_message_recipient_output_attached";
pub const MESSAGE_RECIPIENT_OUTPUT_UNAVAILABLE: &str = "tc_message_recipient_output_unavailable";
pub const MESSAGE_RECIPIENT_OUTPUT_FAILED: &str = "tc_message_recipient_output_failed";

#[derive(Debug, Clone)]
pub struct TcMessageRecipientOutputReport {
    pub status: String,
    pub recipients: Vec<RecipientRecord>,
    pub candidate_count: usize,
    pub failure_reason: Option<String>,
}

/// Builds structured recipient output from one message-attributed Table Context probe.
///
/// The operation preserves the exactly-one-table rule and the existing typed conversion
/// boundary. Unavailable, ambiguous, or invalid evidence exposes no partial recipients.
pub fn build_message_recipient_output(
    message_key: &str,
    probe: &TcSubnodeProbeReport,
) -> TcMessageRecipientOutputReport {
    let selection = select_message_recipient_projection(probe);
    if selection.status != MESSAGE_RECIPIENT_PROJECTION_SELECTED {
        return TcMessageRecipientOutputReport {
            status: MESSAGE_RECIPIENT_OUTPUT_UNAVAILABLE.to_string(),
            recipients: Vec::new(),
            candidate_count: selection.candidate_count,
            failure_reason: selection.failure_reason,
        };
    }

    let Some(complete_records) = selection.complete_records else {
        return failed(
            selection.candidate_count,
            "selected recipient projection did not retain complete records".to_string(),
        );
    };

    let converted = message_recipients_from_complete_records(message_key, &complete_records);
    if converted.status != MESSAGE_RECIPIENTS_ATTACHED {
        return failed(
            selection.candidate_count,
            converted
                .failure_reason
                .unwrap_or_else(|| "recipient output conversion failed".to_string()),
        );
    }

    TcMessageRecipientOutputReport {
        status: MESSAGE_RECIPIENT_OUTPUT_ATTACHED.to_string(),
        recipients: converted.recipients,
        candidate_count: selection.candidate_count,
        failure_reason: None,
    }
}

fn failed(candidate_count: usize, reason: String) -> TcMessageRecipientOutputReport {
    TcMessageRecipientOutputReport {
        status: MESSAGE_RECIPIENT_OUTPUT_FAILED.to_string(),
        recipients: Vec::new(),
        candidate_count,
        failure_reason: Some(reason),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_message_recipient_output, MESSAGE_RECIPIENT_OUTPUT_ATTACHED,
        MESSAGE_RECIPIENT_OUTPUT_UNAVAILABLE,
    };
    use crate::pst::tc_complete_recipient_projection::TcCompleteRecipientProjectionReport;
    use crate::pst::tc_recipient_identity_diagnostic::TcRecipientIdentityDiagnostic;
    use crate::pst::tc_recipient_records::{
        TcCompleteRecipientRecord, TcCompleteRecipientRecordReport, RECIPIENT_RECORDS_VALIDATED,
    };
    use crate::pst::tc_reporting::{TcHeapDiagnostic, TcSubnodeProbeReport, TcTableHeapReport};

    #[test]
    fn builds_four_structured_recipients_from_one_validated_projection() {
        let probe = probe_with_projections(vec![validated_projection("Recipient 1")]);

        let output = build_message_recipient_output("msg_123", &probe);

        assert_eq!(output.status, MESSAGE_RECIPIENT_OUTPUT_ATTACHED);
        assert_eq!(output.candidate_count, 1);
        assert_eq!(output.recipients.len(), 1);
        assert_eq!(output.recipients[0].message_key, "msg_123");
        assert_eq!(output.recipients[0].recipient_type, "to");
        assert_eq!(
            output.recipients[0].raw_address.as_deref(),
            Some("recipient@example.com")
        );
        assert!(output.recipients[0].smtp_address.is_none());
    }

    #[test]
    fn ambiguous_projections_expose_no_partial_recipients() {
        let probe = probe_with_projections(vec![
            validated_projection("Recipient 1"),
            validated_projection("Recipient 2"),
        ]);

        let output = build_message_recipient_output("msg_123", &probe);

        assert_eq!(output.status, MESSAGE_RECIPIENT_OUTPUT_UNAVAILABLE);
        assert_eq!(output.candidate_count, 2);
        assert!(output.recipients.is_empty());
        assert!(output.failure_reason.is_some());
    }

    fn probe_with_projections(
        projections: Vec<TcCompleteRecipientProjectionReport>,
    ) -> TcSubnodeProbeReport {
        let diagnostics = projections
            .into_iter()
            .enumerate()
            .map(|(index, projection)| TcHeapDiagnostic {
                payload_index: index,
                source: "subnode".to_string(),
                status: "resolved".to_string(),
                complete_recipients: Some(projection),
                ..Default::default()
            })
            .collect();

        TcSubnodeProbeReport {
            root_node_id: 1,
            root_subnode_block_id: 2,
            decoded_payload_count: 1,
            table_heaps: TcTableHeapReport {
                status: "resolved".to_string(),
                diagnostics,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn validated_projection(display_name: &str) -> TcCompleteRecipientProjectionReport {
        TcCompleteRecipientProjectionReport {
            display_names: TcRecipientIdentityDiagnostic::unavailable("test"),
            addresses: TcRecipientIdentityDiagnostic::unavailable("test"),
            complete_records: TcCompleteRecipientRecordReport {
                status: RECIPIENT_RECORDS_VALIDATED.to_string(),
                records: vec![TcCompleteRecipientRecord {
                    row_index: 0,
                    role: "to".to_string(),
                    display_name: display_name.to_string(),
                    address: "recipient@example.com".to_string(),
                    address_kind: "native_email_address".to_string(),
                }],
                failure_reason: None,
            },
        }
    }
}
