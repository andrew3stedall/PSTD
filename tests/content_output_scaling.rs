use pstd::config::AttachmentTextMode;
use pstd::output::attachment_store::{retrieve_attachment_by_id, write_disk_attachments};
use pstd::output::attachment_text::{iter_attachment_text_records, parse_attachment_text_records};
use pstd::output::metadata::{MessageRecord, RecipientRecord};
use pstd::output::reconstruction::{build_email_content_records, iter_email_content_records};
use pstd::pst::attachments::{attachment_payload, AttachmentMetadata};
use pstd::pst::messages::{html_body_payload, text_body_payload};
use std::fs;

fn message(key: &str) -> MessageRecord {
    MessageRecord {
        run_id: "run".to_string(),
        pst_id: "pst".to_string(),
        folder_key: "folder".to_string(),
        message_key: key.to_string(),
        message_node_id: None,
        folder_path: "/Inbox".to_string(),
        item_type: "message".to_string(),
        message_class: None,
        subject: Some("Fixture subject".to_string()),
        sender_name: Some("Fixture Sender".to_string()),
        sender_email: Some("sender@example.com".to_string()),
        sender_raw_address: None,
        sender_address_type: Some("SMTP".to_string()),
        sent_representing_email: None,
        sent_representing_address_type: None,
        received_by_email: None,
        received_by_address_type: None,
        received_representing_email: None,
        received_representing_address_type: None,
        sent_at: None,
        received_at: None,
        created_at: None,
        modified_at: None,
        importance: None,
        message_flags: None,
        priority: None,
        sensitivity: None,
        read_receipt_requested: None,
        reply_requested: None,
        delivery_report_requested: None,
        delete_after_submit: None,
        transport_message_headers: Some("Date: 19 Aug 2015 11:07:26 +0000\r\n".to_string()),
        internet_message_id: Some("<fixture@example.com>".to_string()),
        in_reply_to_id: None,
        conversation_index: None,
        conversation_topic: None,
        normalized_subject: None,
        has_text_body: true,
        has_html_body: false,
        has_attachments: false,
        attachment_count: 0,
        metadata_status: "ok".to_string(),
        threading_status: "ok".to_string(),
        body_status: "ok".to_string(),
        attachment_status: "none".to_string(),
        extraction_status: "ok".to_string(),
    }
}

fn recipient(message_key: &str) -> RecipientRecord {
    RecipientRecord {
        message_key: message_key.to_string(),
        recipient_key: format!("recipient-{message_key}"),
        recipient_type: "to".to_string(),
        display_name: Some("Recipient".to_string()),
        raw_address: Some("recipient@example.com".to_string()),
        address_type: Some("native_email_address".to_string()),
        smtp_address: None,
        resolution_status: "validated".to_string(),
        ordinal: 0,
    }
}

#[test]
fn content_indexes_preserve_order_missing_and_payload_only_records() {
    let messages = vec![message("msg_b"), message("msg_a"), message("msg_empty")];
    let text = text_body_payload("msg_a", "Alpha");
    let html = html_body_payload("msg_a", b"<p>Alpha</p>");
    let other = text_body_payload("msg_b", "Beta");
    let mut unavailable = text_body_payload("msg_b", "unavailable").record;
    unavailable.body_key = "missing".into();
    let mut late = recipient("msg_a");
    late.ordinal = 5;
    late.recipient_key = "late".into();
    let recipients = vec![late, recipient("msg_b"), recipient("msg_a")];
    let bodies = vec![unavailable, other.record.clone(), text.record.clone()];
    let payloads = vec![other, html, text];
    let attachment = attachment_payload("msg_a", 0, AttachmentMetadata::default(), vec![]);
    let attachments = vec![attachment.record];
    let actual = iter_email_content_records(
        &messages,
        &[],
        &recipients,
        &bodies,
        &payloads,
        &attachments,
    )
    .collect::<Vec<_>>();
    assert_eq!(
        actual
            .iter()
            .map(|row| row.message.message_key.as_str())
            .collect::<Vec<_>>(),
        ["msg_a", "msg_b", "msg_empty"]
    );
    assert_eq!(
        actual[0]
            .bodies
            .iter()
            .map(|body| body.record.body_type.as_str())
            .collect::<Vec<_>>(),
        ["text", "html"]
    );
    assert_eq!(actual[0].bodies[0].text.as_deref(), Some("Alpha"));
    assert_eq!(
        actual[0]
            .recipients
            .iter()
            .map(|row| row.ordinal)
            .collect::<Vec<_>>(),
        [0, 5]
    );
    assert_eq!(
        actual[0].attachment_ids,
        [attachments[0].attachment_key.clone()]
    );
    assert_eq!(actual[1].bodies.len(), 2);
    assert_eq!(
        actual[1]
            .bodies
            .iter()
            .filter(|body| body.payload_present)
            .count(),
        1
    );
    assert!(actual[2].bodies.is_empty());
    assert_eq!(actual[2].reconstruction_status, "body_payload_unavailable");
    let collected = build_email_content_records(
        &messages,
        &[],
        &recipients,
        &bodies,
        &payloads,
        &attachments,
    );
    assert_eq!(
        serde_json::to_vec(&actual).unwrap(),
        serde_json::to_vec(&collected).unwrap()
    );
}

#[test]
fn content_index_does_not_attach_another_messages_body_on_key_collision() {
    let payload = text_body_payload("msg_other", "Other message bytes");
    let mut record = payload.record.clone();
    record.message_key = "msg_a".into();
    let rows =
        build_email_content_records(&[message("msg_a")], &[], &[], &[record], &[payload], &[]);
    assert!(!rows[0].bodies[0].payload_present);
    assert!(rows[0].bodies[0].raw_bytes_base64.is_none());
}

#[test]
fn attachment_index_keeps_payload_only_records_and_rejects_duplicate_payloads() {
    let payload = attachment_payload("msg_a", 0, AttachmentMetadata::default(), vec![]);
    let mut missing = payload.record.clone();
    missing.attachment_key = "missing".into();
    missing.ordinal = 1;
    let records = parse_attachment_text_records(
        AttachmentTextMode::OfficePdf,
        &[missing],
        std::slice::from_ref(&payload),
    );
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].attachment_key, payload.record.attachment_key);
    assert_eq!(records[1].status, "attachment_payload_unavailable");
    let duplicates = vec![payload.clone(), payload];
    let records = iter_attachment_text_records(AttachmentTextMode::OfficePdf, &[], &duplicates)
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].status, "attachment_payload_duplicate_id");
    assert!(records[0].text.is_none());
    assert_eq!(
        iter_attachment_text_records(AttachmentTextMode::None, &[], &duplicates).count(),
        0
    );
}

#[test]
fn disk_export_preflights_duplicate_ids_and_paths_before_writing() {
    let first = attachment_payload("msg_a", 0, AttachmentMetadata::default(), vec![1]);
    let second = attachment_payload("msg_b", 0, AttachmentMetadata::default(), vec![2]);
    let root = tempfile::tempdir().unwrap();
    assert!(write_disk_attachments(
        root.path(),
        &[],
        &[first.clone(), second.clone(), second.clone()]
    )
    .is_err());
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
    let mut collision = second.record.clone();
    collision.archive_path = first.record.archive_path.clone();
    assert!(write_disk_attachments(root.path(), &[first.record.clone(), collision], &[]).is_err());
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
    let mut same_id = first.record.clone();
    same_id.archive_path = second.record.archive_path.clone();
    assert!(write_disk_attachments(root.path(), &[first.record, same_id], &[]).is_err());
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
}

#[test]
fn retrieval_scans_after_match_for_duplicates_and_malformed_rows_and_validates_bytes() {
    let payload = attachment_payload("msg_a", 0, AttachmentMetadata::default(), vec![1, 2]);
    let root = tempfile::tempdir().unwrap();
    let (manifest, _) =
        write_disk_attachments(root.path(), &[], std::slice::from_ref(&payload)).unwrap();
    let path = root.path().join("attachments.jsonl");
    let line = String::from_utf8(manifest).unwrap();
    // CRLF, whitespace-only rows and an unterminated final row remain accepted.
    fs::write(&path, format!("\r\n  \r\n{}", line.trim_end())).unwrap();
    assert_eq!(
        retrieve_attachment_by_id(root.path(), &payload.record.attachment_key)
            .unwrap()
            .bytes,
        payload.bytes
    );
    fs::write(&path, format!("{line}{line}")).unwrap();
    assert!(
        retrieve_attachment_by_id(root.path(), &payload.record.attachment_key)
            .unwrap_err()
            .to_string()
            .contains("duplicate")
    );
    fs::write(&path, format!("{line}not-json\n")).unwrap();
    assert!(retrieve_attachment_by_id(root.path(), &payload.record.attachment_key).is_err());
    fs::write(&path, &line).unwrap();
    fs::write(root.path().join(&payload.record.archive_path), [9, 9]).unwrap();
    assert!(
        retrieve_attachment_by_id(root.path(), &payload.record.attachment_key)
            .unwrap_err()
            .to_string()
            .contains("validation")
    );
    // Canonical unpacked archive manifest fallback retains the same validation.
    fs::create_dir(root.path().join("data")).unwrap();
    fs::rename(&path, root.path().join("data/attachments.jsonl")).unwrap();
    fs::write(
        root.path().join(&payload.record.archive_path),
        &payload.bytes,
    )
    .unwrap();
    assert_eq!(
        retrieve_attachment_by_id(root.path(), &payload.record.attachment_key)
            .unwrap()
            .bytes,
        payload.bytes
    );
}
