//! Deterministic output-phase benchmark; see scripts/benchmark_content_output.py.
use std::time::Instant;

use pstd::config::AttachmentTextMode;
use pstd::output::attachment_store::write_disk_attachments;
use pstd::output::attachment_text::iter_attachment_text_records;
use pstd::output::jsonl_writer::JsonlBuffer;
use pstd::output::metadata::{MessageRecord, RecipientRecord};
use pstd::output::reconstruction::iter_email_content_records;
use pstd::pst::attachments::{attachment_payload, AttachmentMetadata};
use pstd::pst::messages::text_body_payload;
use sha2::{Digest, Sha256};

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let mode = &args[1];
    let count = args[2].parse::<usize>().unwrap();
    let body_bytes = args[3].parse::<usize>().unwrap();
    let text = "x".repeat(body_bytes);
    let mut messages = Vec::new();
    let mut recipients = Vec::new();
    let mut bodies = Vec::new();
    let mut attachments = Vec::new();
    for index in (0..count).rev() {
        let key = format!("msg_{index:08}");
        messages.push(serde_json::from_value::<MessageRecord>(serde_json::json!({
            "run_id": "run_benchmark", "pst_id": "pst_benchmark", "folder_key": "inbox",
            "message_key": key, "folder_path": "/Inbox", "item_type": "message",
            "subject": "Synthetic benchmark", "has_text_body": true, "has_html_body": false,
            "has_attachments": true, "attachment_count": 1, "metadata_status": "ok",
            "threading_status": "ok", "body_status": "ok", "attachment_status": "ok",
            "extraction_status": "ok"
        })).unwrap());
        recipients.push(RecipientRecord {
            message_key: key.clone(), recipient_key: format!("recipient_{index}"),
            recipient_type: "to".into(), display_name: None,
            raw_address: Some("synthetic@example.test".into()), address_type: None,
            smtp_address: None, resolution_status: "ok".into(), ordinal: 0,
        });
        bodies.push(text_body_payload(&key, &text));
        attachments.push(attachment_payload(&key, 0, AttachmentMetadata {
            filename_original: Some("payload.bin".into()), ..Default::default()
        }, vec![1, 2, 3, 4]));
    }
    let body_records = bodies.iter().map(|body| body.record.clone()).collect::<Vec<_>>();
    let attachment_records = attachments.iter().map(|attachment| attachment.record.clone()).collect::<Vec<_>>();
    let started = Instant::now();
    let mut output = JsonlBuffer::new();
    let bytes = match mode.as_str() {
        "email" => {
            for record in iter_email_content_records(&messages, &[], &recipients, &body_records, &bodies, &attachment_records) {
                output.write_record(&record).unwrap();
            }
            output.into_bytes()
        }
        "text" => {
            for record in iter_attachment_text_records(AttachmentTextMode::OfficePdf, &attachment_records, &attachments) {
                output.write_record(&record).unwrap();
            }
            output.into_bytes()
        }
        "disk" => {
            let root = tempfile::tempdir().unwrap();
            let (bytes, emitted) = write_disk_attachments(root.path(), &attachment_records, &attachments).unwrap();
            assert_eq!(emitted, count);
            bytes
        }
        _ => panic!("expected email, text or disk"),
    };
    let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
    println!("{}", serde_json::json!({
        "mode": mode, "records": count, "body_bytes": body_bytes,
        "elapsed_ms": elapsed_ms, "output_bytes": bytes.len(),
        "output_sha256": hex::encode(Sha256::digest(&bytes))
    }));
}
