use crate::output::ids;
use crate::output::metadata::{BodyRecord, ItemKind, MessageRecord, NonMailRecord};
use crate::pst::item_routing::classify_message_class;

pub fn build_non_mail_records(
    messages: &[MessageRecord],
    bodies: &[BodyRecord],
) -> Vec<NonMailRecord> {
    let mut records = messages
        .iter()
        .filter_map(|message| {
            let classification = classify_message_class(message.message_class.as_deref());
            let (item_kind, readpst_status, pstd_status, status) = match classification.kind {
                Some(ItemKind::Journal) => {
                    let status = if message.subject.is_some()
                        || message.sender_name.is_some()
                        || message.created_at.is_some()
                        || message.modified_at.is_some()
                    {
                        "journal_partial_source_fields"
                    } else {
                        "journal_fields_unavailable"
                    };
                    ("journal", "routed_journal", status, status)
                }
                Some(ItemKind::Task) => (
                    "task",
                    "skipped_unsupported_by_readpst",
                    "typed_non_mail_preserved_no_readpst_renderer",
                    "unsupported_by_readpst_preserved",
                ),
                Some(ItemKind::StickyNote) => (
                    "sticky_note",
                    "skipped_unsupported_by_readpst",
                    "typed_non_mail_preserved_no_readpst_renderer",
                    "unsupported_by_readpst_preserved",
                ),
                Some(ItemKind::Other) => (
                    "other",
                    "skipped_unknown_item_class",
                    "unsupported_by_pstd_unknown_class_preserved",
                    "unknown_class_preserved",
                ),
                None => (
                    "unknown",
                    "unavailable_missing_item_class",
                    "unsupported_by_pstd_missing_class_preserved",
                    "missing_class_preserved",
                ),
                Some(
                    ItemKind::Note
                    | ItemKind::Schedule
                    | ItemKind::Appointment
                    | ItemKind::Contact
                    | ItemKind::Report
                    | ItemKind::Store,
                ) => return None,
            };

            let mut raw_evidence_refs = vec![
                format!("message_record:{}", message.message_key),
                format!(
                    "message_class:{}",
                    message.message_class.as_deref().unwrap_or("<missing>")
                ),
            ];
            raw_evidence_refs.extend(
                bodies
                    .iter()
                    .filter(|body| body.message_key == message.message_key)
                    .map(|body| format!("body_record:{}", body.body_key)),
            );
            raw_evidence_refs.sort();

            Some(NonMailRecord {
                non_mail_key: ids::stable_id("non-mail", &[&message.message_key]),
                message_key: message.message_key.clone(),
                folder_path: message.folder_path.clone(),
                source_node_id: message.message_node_id.clone(),
                message_class: message.message_class.clone(),
                item_kind: item_kind.to_string(),
                summary: message.subject.clone(),
                sender_name: message.sender_name.clone(),
                sender_email: message.sender_email.clone(),
                created_at: message.created_at.clone(),
                modified_at: message.modified_at.clone(),
                readpst_status: readpst_status.to_string(),
                pstd_status: pstd_status.to_string(),
                status: status.to_string(),
                authoritative: false,
                synthetic: false,
                raw_evidence_refs,
            })
        })
        .collect::<Vec<_>>();
    records.sort_by_key(|record| record.non_mail_key.clone());
    records
}

pub fn serialize_vjournals(records: &[NonMailRecord]) -> String {
    let mut output = String::new();
    for record in records
        .iter()
        .filter(|record| record.item_kind == "journal")
    {
        output.push_str("BEGIN:VJOURNAL\r\nVERSION:1.0\r\n");
        output.push_str(&format!(
            "UID:{}\r\n",
            escape_vjournal(&record.non_mail_key)
        ));
        output.push_str("DTSTAMP:19700101T000000Z\r\n");
        output.push_str("X-PSTD-DTSTAMP-SYNTHETIC:TRUE\r\n");
        if let Some(summary) = record.summary.as_deref() {
            output.push_str(&format!("SUMMARY:{}\r\n", escape_vjournal(summary)));
        }
        if let Some(created_at) = record.created_at.as_deref() {
            output.push_str(&format!(
                "X-PSTD-SOURCE-CREATED-AT:{}\r\n",
                escape_vjournal(created_at)
            ));
        }
        if let Some(modified_at) = record.modified_at.as_deref() {
            output.push_str(&format!(
                "X-PSTD-SOURCE-MODIFIED-AT:{}\r\n",
                escape_vjournal(modified_at)
            ));
        }
        output.push_str(&format!(
            "X-PSTD-READPST-STATUS:{}\r\nX-PSTD-PSTD-STATUS:{}\r\nX-PSTD-STATUS:{}\r\nX-PSTD-AUTHORITATIVE:{}\r\nX-PSTD-SYNTHETIC:{}\r\n",
            escape_vjournal(&record.readpst_status),
            escape_vjournal(&record.pstd_status),
            escape_vjournal(&record.status),
            record.authoritative,
            record.synthetic,
        ));
        output.push_str("END:VJOURNAL\r\n");
    }
    output
}

fn escape_vjournal(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace('\r', "")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::{build_non_mail_records, serialize_vjournals};
    use crate::output::metadata::MessageRecord;

    fn message(message_class: Option<&str>, key: &str) -> MessageRecord {
        MessageRecord {
            run_id: "run".to_string(),
            pst_id: "pst".to_string(),
            folder_key: "folder".to_string(),
            message_key: key.to_string(),
            message_node_id: Some(format!("node-{key}")),
            folder_path: "/Mixed".to_string(),
            item_type: "message_metadata".to_string(),
            message_class: message_class.map(str::to_string),
            subject: Some("Journal, task".to_string()),
            sender_name: Some("Source User".to_string()),
            sender_email: Some("source@example.test".to_string()),
            sender_raw_address: Some("source@example.test".to_string()),
            sender_address_type: Some("SMTP".to_string()),
            sent_representing_email: None,
            sent_representing_address_type: None,
            received_by_email: None,
            received_by_address_type: None,
            received_representing_email: None,
            received_representing_address_type: None,
            sent_at: None,
            received_at: None,
            created_at: Some("2020-01-02T03:04:05Z".to_string()),
            modified_at: Some("2020-01-03T03:04:05Z".to_string()),
            importance: None,
            message_flags: None,
            priority: None,
            sensitivity: None,
            read_receipt_requested: None,
            reply_requested: None,
            delivery_report_requested: None,
            delete_after_submit: None,
            transport_message_headers: None,
            internet_message_id: None,
            in_reply_to_id: None,
            conversation_index: None,
            conversation_topic: None,
            normalized_subject: None,
            has_text_body: false,
            has_html_body: false,
            has_attachments: false,
            attachment_count: 0,
            metadata_status: "synthetic_non_mail_source".to_string(),
            threading_status: "not_applicable".to_string(),
            body_status: "not_applicable".to_string(),
            attachment_status: "not_applicable".to_string(),
            extraction_status: "non_mail_source".to_string(),
        }
    }

    #[test]
    fn counts_non_mail_classes_once_and_preserves_status_distinctions() {
        let messages = [
            message(Some("IPM.Activity"), "journal"),
            message(Some("IPM.Task"), "task"),
            message(Some("IPM.StickyNote"), "sticky"),
            message(Some("IPM.FutureType"), "other"),
            message(None, "missing"),
            message(Some("IPM.Note"), "note"),
        ];
        let records = build_non_mail_records(&messages, &[]);
        assert_eq!(records.len(), 5);
        assert_eq!(
            records
                .iter()
                .filter(|record| record.readpst_status == "skipped_unsupported_by_readpst")
                .count(),
            2
        );
        assert!(records.iter().any(|record| {
            record.item_kind == "other"
                && record.pstd_status == "unsupported_by_pstd_unknown_class_preserved"
        }));
        assert!(records.iter().any(|record| {
            record.item_kind == "unknown"
                && record.readpst_status == "unavailable_missing_item_class"
        }));
        assert!(!records.iter().any(|record| record.message_key == "note"));
        assert_eq!(
            records
                .iter()
                .map(|record| record.message_key.as_str())
                .collect::<Vec<_>>()
                .len(),
            5
        );
    }

    #[test]
    fn emits_deterministic_vjournal_with_explicit_provenance_status() {
        let records = build_non_mail_records(&[message(Some("IPM.Journal"), "journal")], &[]);
        let output = serialize_vjournals(&records);
        assert!(output.starts_with("BEGIN:VJOURNAL\r\nVERSION:1.0\r\n"));
        assert!(output.contains("SUMMARY:Journal\\, task\r\n"));
        assert!(output.contains("X-PSTD-READPST-STATUS:routed_journal\r\n"));
        assert!(output.contains("X-PSTD-DTSTAMP-SYNTHETIC:TRUE\r\n"));
        assert_eq!(output, serialize_vjournals(&records));
    }
}
