use pstd::eml::{build_plain_text_eml, materialize_embedded_message_payloads};
use pstd::output::metadata::{AttachmentRecord, MessageRecord, RecipientRecord};
use pstd::pst::attachments::{AttachmentPayload, ATTACH_METHOD_EMBEDDED_MESSAGE};
use pstd::pst::messages::{text_body_payload, BodyPayload};
use sha2::{Digest, Sha256};

fn message(key: &str) -> MessageRecord {
    MessageRecord {
        run_id: "run".to_string(),
        pst_id: "pst".to_string(),
        folder_key: "folder".to_string(),
        message_key: key.to_string(),
        message_node_id: None,
        folder_path: "/Inbox".to_string(),
        item_type: "message".to_string(),
        subject: Some("Fixture subject".to_string()),
        sender_name: Some("Fixture Sender".to_string()),
        sender_email: Some("sender@example.com".to_string()),
        sender_raw_address: None,
        sender_address_type: Some("SMTP".to_string()),
        sent_at: None,
        received_at: None,
        created_at: None,
        modified_at: None,
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

fn method5_attachment(
    attachment_key: &str,
    owner_key: &str,
    child_key: Option<&str>,
) -> AttachmentRecord {
    AttachmentRecord {
        message_key: owner_key.to_string(),
        attachment_key: attachment_key.to_string(),
        filename_original: None,
        filename_safe: "attachment_1".to_string(),
        content_type: None,
        extension: None,
        size_bytes: 0,
        declared_size_bytes: None,
        size_status: "payload_unavailable".to_string(),
        sha256: String::new(),
        is_inline: false,
        content_id: None,
        attachment_method: Some(ATTACH_METHOD_EMBEDDED_MESSAGE),
        embedded_message_key: child_key.map(str::to_string),
        ordinal: 1,
        archive_path: format!("attachments/{owner_key}/{attachment_key}_attachment_1"),
        extraction_status: "embedded_message_payload_deferred".to_string(),
    }
}

fn materialize(
    attachments: &mut [AttachmentRecord],
    messages: &[MessageRecord],
    recipients: &[RecipientRecord],
    bodies: &[BodyPayload],
) -> Vec<AttachmentPayload> {
    let mut payloads = Vec::new();
    materialize_embedded_message_payloads(
        attachments,
        &mut payloads,
        messages,
        recipients,
        bodies,
    );
    payloads
}

#[test]
fn materializes_exact_shared_eml_bytes_and_updates_record() {
    let child_key = "child";
    let child = message(child_key);
    let recipients = vec![recipient(child_key)];
    let bodies = vec![text_body_payload(child_key, "plain\nbody")];
    let expected = build_plain_text_eml(&child, &recipients, &bodies[0].bytes).unwrap();
    let mut attachments = vec![method5_attachment("att-1", "parent", Some(child_key))];

    let payloads = materialize(
        &mut attachments,
        std::slice::from_ref(&child),
        &recipients,
        &bodies,
    );

    assert_eq!(payloads.len(), 1);
    assert_eq!(payloads[0].bytes, expected);
    assert_eq!(payloads[0].record.attachment_key, "att-1");
    assert_eq!(payloads[0].record.message_key, "parent");
    assert_eq!(payloads[0].record.embedded_message_key.as_deref(), Some(child_key));
    assert_eq!(payloads[0].record.content_type.as_deref(), Some("message/rfc822"));
    assert_eq!(payloads[0].record.size_bytes, expected.len() as u64);
    assert_eq!(payloads[0].record.archive_path, attachments[0].archive_path);
    assert_eq!(
        payloads[0].record.extraction_status,
        "extracted_embedded_message_eml"
    );
    assert_eq!(
        payloads[0].record.sha256,
        hex::encode(Sha256::digest(&expected))
    );
    assert_eq!(attachments[0].sha256, payloads[0].record.sha256);
}

#[test]
fn rejects_missing_and_mismatched_child_links() {
    let child = message("child");
    let recipients = vec![recipient("child")];
    let bodies = vec![text_body_payload("child", "plain")];

    for child_key in [None, Some("missing")] {
        let mut attachments = vec![method5_attachment("att-1", "parent", child_key)];
        let payloads = materialize(
            &mut attachments,
            std::slice::from_ref(&child),
            &recipients,
            &bodies,
        );
        assert!(payloads.is_empty());
        assert_eq!(
            attachments[0].extraction_status,
            "embedded_message_payload_deferred"
        );
    }
}

#[test]
fn rejects_duplicate_child_links_and_nested_children() {
    let child = message("child");
    let recipients = vec![recipient("child")];
    let bodies = vec![text_body_payload("child", "plain")];

    let mut duplicate_links = vec![
        method5_attachment("att-1", "parent", Some("child")),
        method5_attachment("att-2", "parent", Some("child")),
    ];
    assert!(materialize(
        &mut duplicate_links,
        std::slice::from_ref(&child),
        &recipients,
        &bodies,
    )
    .is_empty());

    let mut nested = vec![
        method5_attachment("att-1", "parent", Some("child")),
        method5_attachment("nested", "child", None),
    ];
    assert!(materialize(
        &mut nested,
        std::slice::from_ref(&child),
        &recipients,
        &bodies,
    )
    .is_empty());
}

#[test]
fn rejects_ambiguous_body_and_header_injection() {
    let child_key = "child";
    let recipients = vec![recipient(child_key)];
    let mut attachments = vec![method5_attachment("att-1", "parent", Some(child_key))];
    let duplicate_bodies = vec![
        text_body_payload(child_key, "one"),
        text_body_payload(child_key, "two"),
    ];
    assert!(materialize(
        &mut attachments,
        &[message(child_key)],
        &recipients,
        &duplicate_bodies,
    )
    .is_empty());

    let mut unsafe_message = message(child_key);
    unsafe_message.subject = Some("unsafe\r\nBcc: injected@example.com".to_string());
    let bodies = vec![text_body_payload(child_key, "plain")];
    assert!(materialize(
        &mut attachments,
        &[unsafe_message],
        &recipients,
        &bodies,
    )
    .is_empty());
}
