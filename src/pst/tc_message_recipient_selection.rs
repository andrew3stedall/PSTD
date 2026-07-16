use crate::pst::tc_recipient_records::{
    TcCompleteRecipientRecordReport, RECIPIENT_RECORDS_VALIDATED,
};
use crate::pst::tc_reporting::TcSubnodeProbeReport;

pub const MESSAGE_RECIPIENT_PROJECTION_SELECTED: &str = "tc_message_recipient_projection_selected";
pub const MESSAGE_RECIPIENT_PROJECTION_UNAVAILABLE: &str =
    "tc_message_recipient_projection_unavailable";
pub const MESSAGE_RECIPIENT_PROJECTION_AMBIGUOUS: &str =
    "tc_message_recipient_projection_ambiguous";

#[derive(Debug, Clone)]
pub struct TcMessageRecipientSelectionReport {
    pub status: String,
    pub complete_records: Option<TcCompleteRecipientRecordReport>,
    pub candidate_count: usize,
    pub failure_reason: Option<String>,
}

/// Selects exactly one validated complete-recipient projection from a message-attributed
/// Table Context probe.
///
/// Zero validated candidates remain unavailable. More than one validated candidate is
/// ambiguous and fails closed rather than combining recipient tables or guessing which
/// table belongs to the message.
pub fn select_message_recipient_projection(
    probe: &TcSubnodeProbeReport,
) -> TcMessageRecipientSelectionReport {
    select_complete_record_report(
        probe
            .table_heaps
            .diagnostics
            .iter()
            .filter(|diagnostic| {
                probe
                    .direct_recipient_table_bids
                    .contains(&diagnostic.block_id)
            })
            .filter_map(|diagnostic| diagnostic.complete_recipients.as_ref())
            .map(|projection| &projection.complete_records),
    )
}

fn select_complete_record_report<'a>(
    reports: impl Iterator<Item = &'a TcCompleteRecipientRecordReport>,
) -> TcMessageRecipientSelectionReport {
    let mut candidates = reports
        .filter(|report| report.status == RECIPIENT_RECORDS_VALIDATED && !report.records.is_empty())
        .cloned()
        .collect::<Vec<_>>();
    let candidate_count = candidates.len();

    match candidate_count {
        1 => TcMessageRecipientSelectionReport {
            status: MESSAGE_RECIPIENT_PROJECTION_SELECTED.to_string(),
            complete_records: candidates.pop(),
            candidate_count,
            failure_reason: None,
        },
        0 => TcMessageRecipientSelectionReport {
            status: MESSAGE_RECIPIENT_PROJECTION_UNAVAILABLE.to_string(),
            complete_records: None,
            candidate_count,
            failure_reason: Some(
                "no validated complete-recipient projection was attributed to the message"
                    .to_string(),
            ),
        },
        _ => TcMessageRecipientSelectionReport {
            status: MESSAGE_RECIPIENT_PROJECTION_AMBIGUOUS.to_string(),
            complete_records: None,
            candidate_count,
            failure_reason: Some(format!(
                "multiple validated complete-recipient projections were attributed to the message: {candidate_count}"
            )),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        select_complete_record_report, MESSAGE_RECIPIENT_PROJECTION_AMBIGUOUS,
        MESSAGE_RECIPIENT_PROJECTION_SELECTED, MESSAGE_RECIPIENT_PROJECTION_UNAVAILABLE,
    };
    use crate::pst::tc_recipient_records::{
        TcCompleteRecipientRecord, TcCompleteRecipientRecordReport, RECIPIENT_RECORDS_FAILED,
        RECIPIENT_RECORDS_VALIDATED,
    };

    #[test]
    fn selects_exactly_one_validated_non_empty_report() {
        let selected = validated("Recipient 1");
        let failed = failed();

        let report = select_complete_record_report([&failed, &selected].into_iter());

        assert_eq!(report.status, MESSAGE_RECIPIENT_PROJECTION_SELECTED);
        assert_eq!(report.candidate_count, 1);
        assert_eq!(
            report.complete_records.unwrap().records[0].display_name,
            "Recipient 1"
        );
    }

    #[test]
    fn zero_validated_reports_remain_unavailable() {
        let failed = failed();

        let report = select_complete_record_report([&failed].into_iter());

        assert_eq!(report.status, MESSAGE_RECIPIENT_PROJECTION_UNAVAILABLE);
        assert_eq!(report.candidate_count, 0);
        assert!(report.complete_records.is_none());
    }

    #[test]
    fn multiple_validated_reports_fail_closed_without_partial_records() {
        let first = validated("Recipient 1");
        let second = validated("Recipient 2");

        let report = select_complete_record_report([&first, &second].into_iter());

        assert_eq!(report.status, MESSAGE_RECIPIENT_PROJECTION_AMBIGUOUS);
        assert_eq!(report.candidate_count, 2);
        assert!(report.complete_records.is_none());
        assert!(report.failure_reason.is_some());
    }

    fn validated(display_name: &str) -> TcCompleteRecipientRecordReport {
        TcCompleteRecipientRecordReport {
            status: RECIPIENT_RECORDS_VALIDATED.to_string(),
            records: vec![TcCompleteRecipientRecord {
                row_index: 0,
                role: "to".to_string(),
                display_name: display_name.to_string(),
                address: "recipient@example.com".to_string(),
                address_kind: "native_email_address".to_string(),
            }],
            failure_reason: None,
        }
    }

    fn failed() -> TcCompleteRecipientRecordReport {
        TcCompleteRecipientRecordReport {
            status: RECIPIENT_RECORDS_FAILED.to_string(),
            records: Vec::new(),
            failure_reason: Some("unavailable".to_string()),
        }
    }
}
