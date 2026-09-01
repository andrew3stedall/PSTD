use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

use crate::output::ids;
use crate::output::metadata::{BodyRecord, MessageRecord, SpecialItemRecord};
use crate::pst::messages::BodyPayload;
use crate::pst::rtf;

pub fn build_special_items(
    messages: &[MessageRecord],
    bodies: &[BodyRecord],
    body_payloads: &[BodyPayload],
) -> Vec<SpecialItemRecord> {
    let payloads = body_payloads
        .iter()
        .map(|payload| (payload.record.body_key.as_str(), payload))
        .collect::<BTreeMap<_, _>>();
    let mut ordered_messages = messages.iter().collect::<Vec<_>>();
    ordered_messages.sort_by_key(|message| message.message_key.clone());
    let mut records = Vec::new();

    for message in ordered_messages {
        let class = message
            .message_class
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase();
        let is_report = class.starts_with("report.") || class.contains("ndr");
        let is_schedule = class.starts_with("ipm.schedule.")
            || class.contains("calendar")
            || class.contains("meeting");
        let is_encrypted = class.contains("encrypted");
        let message_bodies = bodies
            .iter()
            .filter(|body| body.message_key == message.message_key)
            .collect::<Vec<_>>();

        if is_report {
            append_body_kind(
                &mut records,
                message,
                "report",
                "message/delivery-status",
                &message_bodies,
                "report",
                &payloads,
            );
        }
        if is_schedule {
            append_body_kind(
                &mut records,
                message,
                "schedule",
                "text/calendar",
                &message_bodies,
                "calendar",
                &payloads,
            );
        }
        if is_encrypted {
            let encrypted_bodies = message_bodies
                .iter()
                .copied()
                .filter(|body| {
                    matches!(
                        body.body_type.as_str(),
                        "text" | "html" | "rtf" | "encrypted" | "encrypted_html"
                    )
                })
                .collect::<Vec<_>>();
            if encrypted_bodies.is_empty() {
                records.push(unavailable_record(
                    message,
                    "encrypted",
                    "application/octet-stream",
                    "encrypted_body_unavailable",
                ));
            } else {
                for body in encrypted_bodies {
                    let payload = payloads.get(body.body_key.as_str()).copied();
                    records.push(body_record(
                        message,
                        "encrypted",
                        "application/octet-stream",
                        body,
                        payload,
                        "encrypted_body_opaque",
                        false,
                        false,
                        false,
                    ));
                }
            }
        }

        for body in message_bodies
            .iter()
            .copied()
            .filter(|body| body.body_type == "rtf")
        {
            let payload = payloads.get(body.body_key.as_str()).copied();
            records.push(rtf_record(message, body, payload));
        }
    }

    records.sort_by_key(|record| record.special_key.clone());
    records
}

fn append_body_kind(
    records: &mut Vec<SpecialItemRecord>,
    message: &MessageRecord,
    kind: &str,
    media_type: &str,
    message_bodies: &[&BodyRecord],
    body_type: &str,
    payloads: &BTreeMap<&str, &BodyPayload>,
) {
    let matching = message_bodies
        .iter()
        .copied()
        .filter(|body| body.body_type == body_type)
        .collect::<Vec<_>>();
    if matching.is_empty() {
        records.push(unavailable_record(
            message,
            kind,
            media_type,
            match kind {
                "report" => "report_payload_unavailable",
                _ => "schedule_payload_unavailable",
            },
        ));
        return;
    }
    for body in matching {
        let payload = payloads.get(body.body_key.as_str()).copied();
        let status = match kind {
            "report" => "report_payload_observed_report_type_unavailable",
            _ => "schedule_payload_observed_method_unavailable",
        };
        records.push(body_record(
            message, kind, media_type, body, payload, status, true, true, false,
        ));
    }
}

fn body_record(
    message: &MessageRecord,
    kind: &str,
    media_type: &str,
    body: &BodyRecord,
    payload: Option<&BodyPayload>,
    status: &str,
    decoded: bool,
    authoritative: bool,
    synthetic: bool,
) -> SpecialItemRecord {
    let (raw_size_bytes, raw_sha256) = payload
        .map(|payload| {
            (
                payload.bytes.len() as u64,
                Some(payload.record.sha256.clone()),
            )
        })
        .unwrap_or((0, None));
    let (decoded_size_bytes, decoded_sha256) = if decoded {
        payload
            .map(|payload| {
                (
                    Some(payload.bytes.len() as u64),
                    Some(payload.record.sha256.clone()),
                )
            })
            .unwrap_or((None, None))
    } else {
        (None, None)
    };
    SpecialItemRecord {
        message_key: message.message_key.clone(),
        special_key: ids::stable_id("special", &[&message.message_key, kind, &body.body_key]),
        kind: kind.to_string(),
        message_class: message.message_class.clone(),
        media_type: media_type.to_string(),
        parameters: None,
        method: None,
        report_type: None,
        source_body_key: Some(body.body_key.clone()),
        raw_size_bytes,
        raw_sha256,
        decoded_size_bytes,
        decoded_sha256,
        status: if payload.is_some() {
            status.to_string()
        } else {
            format!("{status}; source_body_unavailable")
        },
        authoritative: authoritative && payload.is_some(),
        synthetic,
    }
}

fn unavailable_record(
    message: &MessageRecord,
    kind: &str,
    media_type: &str,
    status: &str,
) -> SpecialItemRecord {
    SpecialItemRecord {
        message_key: message.message_key.clone(),
        special_key: ids::stable_id("special", &[&message.message_key, kind, "unavailable"]),
        kind: kind.to_string(),
        message_class: message.message_class.clone(),
        media_type: media_type.to_string(),
        parameters: None,
        method: None,
        report_type: None,
        source_body_key: None,
        raw_size_bytes: 0,
        raw_sha256: None,
        decoded_size_bytes: None,
        decoded_sha256: None,
        status: status.to_string(),
        authoritative: false,
        synthetic: false,
    }
}

fn rtf_record(
    message: &MessageRecord,
    body: &BodyRecord,
    payload: Option<&BodyPayload>,
) -> SpecialItemRecord {
    let Some(payload) = payload else {
        return SpecialItemRecord {
            message_key: message.message_key.clone(),
            special_key: ids::stable_id(
                "special",
                &[&message.message_key, "rtf_synthetic", &body.body_key],
            ),
            kind: "rtf_synthetic".to_string(),
            message_class: message.message_class.clone(),
            media_type: "application/rtf".to_string(),
            parameters: None,
            method: None,
            report_type: None,
            source_body_key: Some(body.body_key.clone()),
            raw_size_bytes: 0,
            raw_sha256: None,
            decoded_size_bytes: None,
            decoded_sha256: None,
            status: "synthetic_rtf_attachment_source_unavailable".to_string(),
            authoritative: false,
            synthetic: true,
        };
    };
    let validation = rtf::validate(&payload.bytes);
    let decoded = validation.decoded.as_deref();
    SpecialItemRecord {
        message_key: message.message_key.clone(),
        special_key: ids::stable_id(
            "special",
            &[&message.message_key, "rtf_synthetic", &body.body_key],
        ),
        kind: "rtf_synthetic".to_string(),
        message_class: message.message_class.clone(),
        media_type: "application/rtf".to_string(),
        parameters: None,
        method: None,
        report_type: None,
        source_body_key: Some(body.body_key.clone()),
        raw_size_bytes: payload.bytes.len() as u64,
        raw_sha256: Some(payload.record.sha256.clone()),
        decoded_size_bytes: decoded.map(|bytes| bytes.len() as u64),
        decoded_sha256: decoded.map(sha256_hex),
        status: if decoded.is_some() {
            "synthetic_rtf_attachment_available"
        } else {
            "synthetic_rtf_attachment_invalid"
        }
        .to_string(),
        authoritative: false,
        synthetic: true,
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::build_special_items;
    use crate::output::metadata::MessageRecord;
    use crate::pst::messages::text_body_payload;

    fn message(class: Option<&str>) -> MessageRecord {
        MessageRecord {
            run_id: "run".to_string(),
            pst_id: "pst".to_string(),
            folder_key: "folder".to_string(),
            message_key: "msg".to_string(),
            message_node_id: Some("node".to_string()),
            folder_path: "/".to_string(),
            item_type: "message_metadata".to_string(),
            message_class: class.map(ToString::to_string),
            subject: None,
            sender_name: None,
            sender_email: None,
            sender_raw_address: None,
            sender_address_type: None,
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
            transport_message_headers: None,
            internet_message_id: None,
            in_reply_to_id: None,
            conversation_index: None,
            conversation_topic: None,
            normalized_subject: None,
            has_text_body: true,
            has_html_body: false,
            has_attachments: false,
            attachment_count: 0,
            metadata_status: "test".to_string(),
            threading_status: "test".to_string(),
            body_status: "test".to_string(),
            attachment_status: "test".to_string(),
            extraction_status: "test".to_string(),
        }
    }

    #[test]
    fn keeps_report_and_schedule_payloads_explicit_when_unavailable() {
        let messages = vec![
            message(Some("REPORT.IPM.Note")),
            message(Some("IPM.Schedule.Meeting.Request")),
        ];
        let items = build_special_items(&messages, &[], &[]);
        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|item| !item.authoritative));
        assert!(items.iter().any(|item| item.kind == "report"));
        assert!(items.iter().any(|item| item.kind == "schedule"));
    }

    #[test]
    fn keeps_encrypted_bytes_opaque() {
        let payload = text_body_payload("msg", "secret");
        let items = build_special_items(
            &[message(Some("IPM.Note.Encrypted"))],
            std::slice::from_ref(&payload.record),
            std::slice::from_ref(&payload),
        );
        let encrypted = items.iter().find(|item| item.kind == "encrypted").unwrap();
        assert!(!encrypted.authoritative);
        assert!(encrypted.raw_sha256.is_some());
        assert!(encrypted.decoded_sha256.is_none());
    }

    #[test]
    fn preserves_encrypted_property_payload_as_opaque_special_item() {
        let payload = crate::pst::messages::body_payload(
            "msg",
            "encrypted_html",
            b"opaque encrypted bytes".to_vec(),
            None,
        );
        let items = build_special_items(
            &[message(Some("IPM.Note.Encrypted"))],
            std::slice::from_ref(&payload.record),
            std::slice::from_ref(&payload),
        );
        let encrypted = items.iter().find(|item| item.kind == "encrypted").unwrap();
        assert!(!encrypted.authoritative);
        assert_eq!(encrypted.raw_size_bytes, 22);
        assert_eq!(encrypted.status, "encrypted_body_opaque");
        assert_eq!(
            encrypted.source_body_key.as_deref(),
            Some(payload.record.body_key.as_str())
        );
    }

    #[test]
    fn marks_valid_rtf_as_synthetic_without_promoting_it() {
        let payload =
            crate::pst::messages::body_payload("msg", "rtf", b"{\\rtf1\\ansi text}".to_vec(), None);
        let items = build_special_items(
            &[message(None)],
            std::slice::from_ref(&payload.record),
            std::slice::from_ref(&payload),
        );
        let rtf = items
            .iter()
            .find(|item| item.kind == "rtf_synthetic")
            .unwrap();
        assert!(rtf.synthetic);
        assert!(!rtf.authoritative);
        assert_eq!(rtf.status, "synthetic_rtf_attachment_available");
    }
}
