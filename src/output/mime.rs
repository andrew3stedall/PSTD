use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

use crate::output::ids;
use crate::output::metadata::{AttachmentRecord, BodyRecord, MessageRecord, MimePartRecord};
use crate::pst::attachments::AttachmentPayload;
use crate::pst::messages::BodyPayload;
use crate::pst::rtf;

#[derive(Debug, Clone)]
struct BodyPartSpec {
    record: BodyRecord,
    source_body_key: Option<String>,
    media_type: String,
    parameters: Option<String>,
    transfer_encoding: Option<String>,
    raw_size_bytes: u64,
    raw_sha256: Option<String>,
    decoded_size_bytes: Option<u64>,
    decoded_sha256: Option<String>,
    status: String,
    authoritative: bool,
}

pub fn build_mime_parts(
    messages: &[MessageRecord],
    bodies: &[BodyRecord],
    body_payloads: &[BodyPayload],
    attachments: &[AttachmentRecord],
    attachment_payloads: &[AttachmentPayload],
) -> Vec<MimePartRecord> {
    let body_payloads = body_payloads
        .iter()
        .map(|payload| (payload.record.body_key.as_str(), payload))
        .collect::<BTreeMap<_, _>>();
    let attachment_payloads = attachment_payloads
        .iter()
        .map(|payload| (payload.record.attachment_key.as_str(), payload))
        .collect::<BTreeMap<_, _>>();
    let mut ordered_messages = messages.iter().collect::<Vec<_>>();
    ordered_messages.sort_by(|left, right| left.message_key.cmp(&right.message_key));

    let mut parts = Vec::new();
    for message in ordered_messages {
        let mut body_records = bodies
            .iter()
            .filter(|body| body.message_key == message.message_key)
            .cloned()
            .collect::<Vec<_>>();
        body_records.sort_by_key(|body| (body_order(&body.body_type), body.body_key.clone()));
        let mut body_specs = body_records
            .iter()
            .map(|body| body_spec(body, body_payloads.get(body.body_key.as_str()).copied()))
            .collect::<Vec<_>>();
        if !body_specs
            .iter()
            .any(|spec| spec.record.body_type == "html")
        {
            if let Some(rtf_spec) = body_specs
                .iter()
                .find(|spec| spec.record.body_type == "rtf")
                .cloned()
            {
                if let Some(payload) = body_payloads.get(rtf_spec.record.body_key.as_str()) {
                    let validation = rtf::validate(&payload.bytes);
                    if let Some(html) = validation.recovered_html {
                        body_specs.push(derived_html_spec(&rtf_spec, html.as_bytes()));
                    }
                }
            }
        }

        let mut attachment_records = attachments
            .iter()
            .filter(|attachment| attachment.message_key == message.message_key)
            .cloned()
            .collect::<Vec<_>>();
        attachment_records.sort_by_key(|attachment| {
            (
                attachment.rendering_position.unwrap_or(attachment.ordinal),
                attachment.attachment_key.clone(),
            )
        });

        let message_class = message
            .message_class
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase();
        let is_report = message_class.starts_with("report.") || message_class.contains("ndr");
        let is_schedule = message_class.starts_with("ipm.schedule.")
            || message_class.contains("calendar")
            || message_class.contains("meeting");
        let is_encrypted = message_class.contains("encrypted");
        if is_encrypted {
            for spec in &mut body_specs {
                spec.media_type = "application/octet-stream".to_string();
                spec.parameters = None;
                spec.transfer_encoding = Some("base64".to_string());
                spec.decoded_size_bytes = None;
                spec.decoded_sha256 = None;
                spec.status = format!("encrypted_body_opaque; source_status={}", spec.status);
                spec.authoritative = false;
            }
        }
        let needs_mixed_root =
            !attachment_records.is_empty() || is_report || is_schedule || is_encrypted;
        let has_text_and_html = body_specs
            .iter()
            .any(|spec| spec.record.body_type == "text")
            && body_specs
                .iter()
                .any(|spec| spec.record.body_type == "html");
        let needs_alternative_root = has_text_and_html && !needs_mixed_root;
        let needs_root = needs_mixed_root || needs_alternative_root || body_specs.len() != 1;

        let root_key = ids::stable_id("mime", &[&message.message_key, "root"]);
        let root_kind = if is_report {
            "multipart_report"
        } else if needs_mixed_root {
            "multipart_mixed"
        } else if needs_alternative_root {
            "multipart_alternative"
        } else {
            "body"
        };
        let root_media_type = match root_kind {
            "multipart_report" => "multipart/report",
            "multipart_mixed" => "multipart/mixed",
            "multipart_alternative" => "multipart/alternative",
            _ => body_specs
                .first()
                .map(|spec| spec.media_type.as_str())
                .unwrap_or("application/octet-stream"),
        };
        let root_parameters = match root_kind {
            "multipart_report" => None,
            "multipart_mixed" => Some("boundary=pstd-mixed".to_string()),
            "multipart_alternative" => Some("boundary=pstd-alternative".to_string()),
            _ => body_specs.first().and_then(|spec| spec.parameters.clone()),
        };
        let root_status = if is_report {
            "mime_report_projection; report_payload_requires_typed_item_properties"
        } else if is_schedule {
            "mime_schedule_projection; calendar_payload_requires_typed_item_properties"
        } else if is_encrypted {
            "mime_encrypted_projection; body_interpretation_forbidden"
        } else if body_specs.is_empty() {
            "mime_body_unavailable"
        } else {
            "mime_projection_complete"
        };
        let root_authoritative =
            !is_report && !is_schedule && !is_encrypted && !body_specs.is_empty();
        if needs_root {
            parts.push(part(
                &message.message_key,
                &root_key,
                None,
                0,
                root_kind,
                Some(root_media_type.to_string()),
                root_parameters,
                None,
                None,
                None,
                None,
                None,
                None,
                0,
                None,
                None,
                None,
                root_status,
                root_authoritative,
                false,
            ));
        }

        let container_key = if needs_root {
            Some(root_key.as_str())
        } else {
            None
        };
        let mut next_ordinal = 0u64;
        if needs_mixed_root && has_text_and_html {
            let alternative_key = ids::stable_id("mime", &[&message.message_key, "alternative"]);
            parts.push(part(
                &message.message_key,
                &alternative_key,
                container_key,
                next_ordinal,
                "multipart_alternative",
                Some("multipart/alternative".to_string()),
                Some("boundary=pstd-alternative".to_string()),
                None,
                None,
                None,
                None,
                None,
                None,
                0,
                None,
                None,
                None,
                "mime_alternative_projection",
                true,
                false,
            ));
            next_ordinal += 1;
            for spec in body_specs
                .iter()
                .filter(|spec| matches!(spec.record.body_type.as_str(), "text" | "html"))
            {
                push_body_part(
                    &mut parts,
                    &message.message_key,
                    Some(&alternative_key),
                    next_ordinal,
                    spec,
                );
                next_ordinal += 1;
            }
            for spec in body_specs
                .iter()
                .filter(|spec| !matches!(spec.record.body_type.as_str(), "text" | "html"))
            {
                push_body_part(
                    &mut parts,
                    &message.message_key,
                    container_key,
                    next_ordinal,
                    spec,
                );
                next_ordinal += 1;
            }
        } else {
            if !needs_root && body_specs.len() == 1 {
                push_body_part(
                    &mut parts,
                    &message.message_key,
                    None,
                    next_ordinal,
                    &body_specs[0],
                );
            } else {
                for spec in &body_specs {
                    push_body_part(
                        &mut parts,
                        &message.message_key,
                        container_key,
                        next_ordinal,
                        spec,
                    );
                    next_ordinal += 1;
                }
            }
        }

        if is_report
            && !body_specs
                .iter()
                .any(|spec| spec.record.body_type == "report")
        {
            parts.push(part(
                &message.message_key,
                &ids::stable_id("mime", &[&message.message_key, "report-body"]),
                Some(container_key.unwrap_or(&root_key)),
                next_ordinal,
                "report_body",
                Some("message/delivery-status".to_string()),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                0,
                None,
                None,
                None,
                "report_body_unavailable_non_authoritative",
                false,
                false,
            ));
            next_ordinal += 1;
        }
        if is_schedule {
            parts.push(part(
                &message.message_key,
                &ids::stable_id("mime", &[&message.message_key, "calendar"]),
                Some(container_key.unwrap_or(&root_key)),
                next_ordinal,
                "calendar",
                Some("text/calendar".to_string()),
                Some("charset=utf-8".to_string()),
                None,
                Some("attachment".to_string()),
                None,
                None,
                None,
                None,
                0,
                None,
                None,
                None,
                "schedule_payload_unavailable_non_authoritative",
                false,
                true,
            ));
            next_ordinal += 1;
        }

        if !is_encrypted {
            for spec in body_specs
                .iter()
                .filter(|spec| spec.record.body_type == "rtf")
            {
                let status = if spec.authoritative {
                    "synthetic_rtf_attachment_available"
                } else {
                    "synthetic_rtf_attachment_unavailable"
                };
                parts.push(part(
                    &message.message_key,
                    &ids::stable_id(
                        "mime",
                        &[&message.message_key, "synthetic-rtf", &spec.record.body_key],
                    ),
                    Some(container_key.unwrap_or(&root_key)),
                    next_ordinal,
                    "synthetic_rtf_attachment",
                    Some("application/rtf".to_string()),
                    None,
                    Some("base64".to_string()),
                    Some("attachment".to_string()),
                    None,
                    Some(spec.record.body_key.clone()),
                    None,
                    None,
                    spec.raw_size_bytes,
                    spec.raw_sha256.clone(),
                    spec.decoded_size_bytes,
                    spec.decoded_sha256.clone(),
                    status,
                    false,
                    true,
                ));
                next_ordinal += 1;
            }
        }

        for attachment in &attachment_records {
            let payload = attachment_payloads
                .get(attachment.attachment_key.as_str())
                .copied();
            let is_embedded = attachment.attachment_method == Some(5);
            let part_type = if is_embedded {
                "embedded_message"
            } else {
                "attachment"
            };
            let media_type = attachment
                .content_type
                .as_deref()
                .map(safe_media_type)
                .or_else(|| is_embedded.then(|| "message/rfc822".to_string()))
                .or_else(|| Some("application/octet-stream".to_string()));
            let authoritative = payload.is_some()
                && payload
                    .is_some_and(|value| value.record.extraction_status.starts_with("extracted"));
            let status = if authoritative {
                "mime_attachment_payload_linked"
            } else if is_embedded {
                "mime_embedded_message_payload_unavailable"
            } else {
                "mime_attachment_payload_unavailable"
            };
            let (raw_size, raw_sha) = payload
                .map(|value| (value.bytes.len() as u64, Some(value.record.sha256.clone())))
                .unwrap_or((attachment.size_bytes, Some(attachment.sha256.clone())));
            let part_key = ids::stable_id(
                "mime",
                &[
                    &message.message_key,
                    "attachment",
                    &attachment.attachment_key,
                ],
            );
            parts.push(part(
                &message.message_key,
                &part_key,
                Some(container_key.unwrap_or(&root_key)),
                next_ordinal,
                part_type,
                media_type,
                None,
                Some("base64".to_string()),
                Some(
                    if attachment.is_inline {
                        "inline"
                    } else {
                        "attachment"
                    }
                    .to_string(),
                ),
                attachment.content_id.clone(),
                None,
                Some(attachment.attachment_key.clone()),
                attachment.embedded_message_key.clone(),
                raw_size,
                raw_sha,
                payload.map(|value| value.bytes.len() as u64),
                payload.map(|value| value.record.sha256.clone()),
                status,
                authoritative,
                false,
            ));
            next_ordinal += 1;
        }
    }
    parts
}

fn body_spec(body: &BodyRecord, payload: Option<&BodyPayload>) -> BodyPartSpec {
    let media_type = match body.body_type.as_str() {
        "text" => "text/plain",
        "html" => "text/html",
        "rtf" => "application/rtf",
        "report" => "message/delivery-status",
        "calendar" => "text/calendar",
        _ => "application/octet-stream",
    }
    .to_string();
    let parameters = matches!(body.body_type.as_str(), "text" | "html" | "rtf")
        .then(|| "charset=utf-8".to_string());
    let Some(payload) = payload else {
        return BodyPartSpec {
            record: body.clone(),
            source_body_key: None,
            media_type,
            parameters,
            transfer_encoding: None,
            raw_size_bytes: body.size_bytes,
            raw_sha256: Some(body.sha256.clone()),
            decoded_size_bytes: None,
            decoded_sha256: None,
            status: format!("body_source_unavailable; {}", body.status),
            authoritative: false,
        };
    };

    if body.body_type == "rtf" {
        let validation = rtf::validate(&payload.bytes);
        let decoded = validation.decoded.as_deref();
        return BodyPartSpec {
            record: body.clone(),
            source_body_key: None,
            media_type,
            parameters,
            transfer_encoding: Some("8bit".to_string()),
            raw_size_bytes: payload.bytes.len() as u64,
            raw_sha256: Some(payload.record.sha256.clone()),
            decoded_size_bytes: decoded.map(|bytes| bytes.len() as u64),
            decoded_sha256: decoded.map(sha256_hex),
            status: validation.status,
            authoritative: decoded.is_some(),
        };
    }

    let transfer_encoding = Some(text_transfer_encoding(&payload.bytes));
    BodyPartSpec {
        record: body.clone(),
        source_body_key: None,
        media_type,
        parameters,
        transfer_encoding,
        raw_size_bytes: payload.bytes.len() as u64,
        raw_sha256: Some(payload.record.sha256.clone()),
        decoded_size_bytes: Some(payload.bytes.len() as u64),
        decoded_sha256: Some(payload.record.sha256.clone()),
        status: body.status.clone(),
        authoritative: true,
    }
}

fn derived_html_spec(rtf_spec: &BodyPartSpec, html: &[u8]) -> BodyPartSpec {
    let message_key = &rtf_spec.record.message_key;
    let body_key = ids::stable_id("body", &[message_key, "rtf-derived-html"]);
    let sha256 = sha256_hex(html);
    BodyPartSpec {
        record: BodyRecord {
            message_key: message_key.clone(),
            body_key,
            body_type: "html".to_string(),
            archive_path: format!("bodies/{message_key}.rtf-derived.html"),
            encoding: Some("utf-8".to_string()),
            size_bytes: html.len() as u64,
            sha256: sha256.clone(),
            status: "derived_from_validated_rtf".to_string(),
        },
        source_body_key: Some(rtf_spec.record.body_key.clone()),
        media_type: "text/html".to_string(),
        parameters: Some("charset=utf-8".to_string()),
        transfer_encoding: Some(text_transfer_encoding(html)),
        raw_size_bytes: rtf_spec.raw_size_bytes,
        raw_sha256: rtf_spec.raw_sha256.clone(),
        decoded_size_bytes: Some(html.len() as u64),
        decoded_sha256: Some(sha256),
        status: "derived_html_from_validated_rtf".to_string(),
        authoritative: true,
    }
}

fn push_body_part(
    parts: &mut Vec<MimePartRecord>,
    message_key: &str,
    parent_key: Option<&str>,
    ordinal: u64,
    spec: &BodyPartSpec,
) {
    let part_key = ids::stable_id("mime", &[message_key, "body", &spec.record.body_key]);
    parts.push(part(
        message_key,
        &part_key,
        parent_key,
        ordinal,
        "body",
        Some(spec.media_type.clone()),
        spec.parameters.clone(),
        spec.transfer_encoding.clone(),
        Some("inline".to_string()),
        None,
        Some(
            spec.source_body_key
                .clone()
                .unwrap_or_else(|| spec.record.body_key.clone()),
        ),
        None,
        None,
        spec.raw_size_bytes,
        spec.raw_sha256.clone(),
        spec.decoded_size_bytes,
        spec.decoded_sha256.clone(),
        &spec.status,
        spec.authoritative,
        false,
    ));
}

#[allow(clippy::too_many_arguments)]
fn part(
    message_key: &str,
    part_key: &str,
    parent_part_key: Option<&str>,
    ordinal: u64,
    part_type: &str,
    media_type: Option<String>,
    parameters: Option<String>,
    transfer_encoding: Option<String>,
    disposition: Option<String>,
    content_id: Option<String>,
    source_body_key: Option<String>,
    source_attachment_key: Option<String>,
    child_message_key: Option<String>,
    raw_size_bytes: u64,
    raw_sha256: Option<String>,
    decoded_size_bytes: Option<u64>,
    decoded_sha256: Option<String>,
    status: &str,
    authoritative: bool,
    synthetic: bool,
) -> MimePartRecord {
    MimePartRecord {
        message_key: message_key.to_string(),
        part_key: part_key.to_string(),
        parent_part_key: parent_part_key.map(ToString::to_string),
        ordinal,
        part_type: part_type.to_string(),
        media_type,
        parameters,
        transfer_encoding,
        disposition,
        content_id,
        source_body_key,
        source_attachment_key,
        child_message_key,
        raw_size_bytes,
        raw_sha256,
        decoded_size_bytes,
        decoded_sha256,
        status: status.to_string(),
        authoritative,
        synthetic,
    }
}

fn body_order(body_type: &str) -> u8 {
    match body_type {
        "text" => 0,
        "html" => 1,
        "rtf" => 2,
        "report" => 3,
        "calendar" => 4,
        _ => 5,
    }
}

fn safe_media_type(value: &str) -> String {
    let value = value.trim();
    if value.contains(';') || value.contains('\r') || value.contains('\n') || !value.contains('/')
    {
        "application/octet-stream".to_string()
    } else {
        value.to_string()
    }
}

fn text_transfer_encoding(bytes: &[u8]) -> String {
    if std::str::from_utf8(bytes).is_ok() && !bytes.contains(&0) {
        "8bit".to_string()
    } else {
        "base64".to_string()
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::build_mime_parts;
    use crate::output::metadata::MessageRecord;
    use crate::pst::attachments::{attachment_payload, AttachmentMetadata};
    use crate::pst::messages::{body_payload, text_body_payload};

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
    fn builds_stable_alternative_and_preserves_raw_hashes() {
        let plain = text_body_payload("msg", "plain");
        let html = body_payload("msg", "html", b"<b>rich</b>".to_vec(), Some("utf-8"));
        let bodies = vec![plain.record.clone(), html.record.clone()];
        let parts = build_mime_parts(
            &[message(None)],
            &bodies,
            &[plain.clone(), html.clone()],
            &[],
            &[],
        );
        assert_eq!(parts[0].part_type, "multipart_alternative");
        assert_eq!(
            parts[0].media_type.as_deref(),
            Some("multipart/alternative")
        );
        assert_eq!(
            parts[1].source_body_key.as_deref(),
            Some(plain.record.body_key.as_str())
        );
        assert_eq!(
            parts[2].source_body_key.as_deref(),
            Some(html.record.body_key.as_str())
        );
        assert!(parts
            .iter()
            .all(|part| part.status != "mime_duplicate_header"));
    }

    #[test]
    fn keeps_unavailable_report_and_schedule_explicit() {
        let parts = build_mime_parts(&[message(Some("REPORT.IPM.Note"))], &[], &[], &[], &[]);
        assert_eq!(parts[0].media_type.as_deref(), Some("multipart/report"));
        assert!(!parts[1].authoritative);
        assert_eq!(parts[1].part_type, "report_body");
    }

    #[test]
    fn orders_attachment_and_embedded_parts_by_rendering_position() {
        let mut first = attachment_payload(
            "msg",
            0,
            AttachmentMetadata {
                filename_original: Some("first.bin".to_string()),
                attachment_method: Some(1),
                ..AttachmentMetadata::default()
            },
            b"one".to_vec(),
        );
        first.record.rendering_position = Some(0);
        let mut second = attachment_payload(
            "msg",
            1,
            AttachmentMetadata {
                filename_original: Some("second.bin".to_string()),
                attachment_method: Some(5),
                ..AttachmentMetadata::default()
            },
            b"two".to_vec(),
        );
        second.record.rendering_position = Some(1);
        let parts = build_mime_parts(
            &[message(None)],
            &[text_body_payload("msg", "body").record.clone()],
            &[text_body_payload("msg", "body")],
            &[first.record.clone(), second.record.clone()],
            &[first, second],
        );
        let attachment_parts = parts
            .iter()
            .filter(|part| part.part_type == "attachment" || part.part_type == "embedded_message")
            .collect::<Vec<_>>();
        assert_eq!(attachment_parts.len(), 2);
        assert_eq!(attachment_parts[0].part_type, "attachment");
        assert_eq!(attachment_parts[1].part_type, "embedded_message");
    }

    #[test]
    fn attachment_mime_tags_use_safe_defaults() {
        let tagged = attachment_payload(
            "msg",
            0,
            AttachmentMetadata {
                filename_original: Some("report.pdf".to_string()),
                content_type: Some("application/pdf".to_string()),
                ..AttachmentMetadata::default()
            },
            b"pdf".to_vec(),
        );
        let missing = attachment_payload(
            "msg",
            1,
            AttachmentMetadata {
                filename_original: Some("blob.bin".to_string()),
                ..AttachmentMetadata::default()
            },
            b"binary".to_vec(),
        );
        let invalid = attachment_payload(
            "msg",
            2,
            AttachmentMetadata {
                filename_original: Some("unsafe.bin".to_string()),
                content_type: Some("text/plain\r\nX-Injected: yes".to_string()),
                ..AttachmentMetadata::default()
            },
            b"unsafe".to_vec(),
        );
        let parts = build_mime_parts(
            &[message(None)],
            &[text_body_payload("msg", "body").record.clone()],
            &[text_body_payload("msg", "body")],
            &[
                tagged.record.clone(),
                missing.record.clone(),
                invalid.record.clone(),
            ],
            &[tagged, missing, invalid],
        );
        let attachment_parts = parts
            .iter()
            .filter(|part| part.part_type == "attachment")
            .collect::<Vec<_>>();
        assert_eq!(
            attachment_parts
                .iter()
                .map(|part| part.media_type.as_deref())
                .collect::<Vec<_>>(),
            vec![
                Some("application/pdf"),
                Some("application/octet-stream"),
                Some("application/octet-stream"),
            ]
        );
        assert!(attachment_parts
            .iter()
            .all(|part| !part.status.contains("Injected")));
    }
}
