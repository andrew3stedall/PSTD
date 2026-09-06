//! Canonical email-content rows for downstream EML reconstruction.
//!
//! The rows deliberately keep attachment bytes out of the email record. Each
//! attachment is represented by its stable key and full metadata, while body
//! bytes are embedded as base64 so the email record is self-contained and
//! lossless for the non-attachment portion of a message.

use std::collections::{BTreeMap, BTreeSet};

use crate::output::metadata::{
    AttachmentRecord, BodyRecord, HeaderProjectionRecord, MessageRecord, RecipientRecord,
};
use crate::pst::attachments::AttachmentPayload;
use crate::pst::messages::BodyPayload;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmailContentRecord {
    pub record_type: String,
    pub schema_version: u32,
    #[serde(flatten)]
    pub message: MessageRecord,
    pub header_projection: Option<HeaderProjectionRecord>,
    pub recipients: Vec<RecipientRecord>,
    pub bodies: Vec<EmailBodyContent>,
    pub attachment_ids: Vec<String>,
    pub attachments: Vec<EmailAttachmentReference>,
    pub reconstruction_status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmailBodyContent {
    #[serde(flatten)]
    pub record: BodyRecord,
    pub payload_present: bool,
    pub raw_bytes_base64: Option<String>,
    pub decoded_bytes_base64: Option<String>,
    pub text: Option<String>,
    pub rendered_html: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmailAttachmentReference {
    #[serde(flatten)]
    pub record: AttachmentRecord,
}

pub fn build_email_content_records(
    messages: &[MessageRecord],
    headers: &[HeaderProjectionRecord],
    recipients: &[RecipientRecord],
    bodies: &[BodyRecord],
    body_payloads: &[BodyPayload],
    attachments: &[AttachmentRecord],
) -> Vec<EmailContentRecord> {
    iter_email_content_records(messages, headers, recipients, bodies, body_payloads, attachments)
        .collect()
}

/// Build one owned record at a time using borrowed, per-message indexes.
/// Consumers can serialize and release each expanded body before building the next.
pub fn iter_email_content_records<'a>(
    messages: &'a [MessageRecord],
    headers: &'a [HeaderProjectionRecord],
    recipients: &'a [RecipientRecord],
    bodies: &'a [BodyRecord],
    body_payloads: &'a [BodyPayload],
    attachments: &'a [AttachmentRecord],
) -> impl Iterator<Item = EmailContentRecord> + 'a {
    let body_payloads = body_payloads
        .iter()
        .map(|payload| (payload.record.body_key.as_str(), payload))
        .collect::<BTreeMap<_, _>>();
    let mut payloads_by_message = BTreeMap::<&str, Vec<&BodyPayload>>::new();
    for payload in body_payloads.values() {
        payloads_by_message
            .entry(payload.record.message_key.as_str())
            .or_default()
            .push(*payload);
    }
    let bodies_by_message = group_by(bodies, |body| body.message_key.as_str());
    let recipients_by_message = group_by(recipients, |recipient| recipient.message_key.as_str());
    let attachments_by_message = group_by(attachments, |attachment| attachment.message_key.as_str());
    let mut headers_by_message = BTreeMap::new();
    for header in headers {
        // Preserve the existing first-header selection and stable input ordering.
        headers_by_message.entry(header.message_key.as_str()).or_insert(header);
    }
    let mut output_messages = messages.iter().collect::<Vec<_>>();
    output_messages.sort_by(|left, right| left.message_key.cmp(&right.message_key));

    output_messages.into_iter().map(move |message| {
        let key = message.message_key.as_str();
        let mut message_bodies = bodies_by_message.get(key).cloned().unwrap_or_default();
        let mut body_keys = message_bodies.iter().map(|body| body.body_key.as_str()).collect::<BTreeSet<_>>();
        for payload in payloads_by_message.get(key).into_iter().flatten() {
            if body_keys.insert(payload.record.body_key.as_str()) {
                message_bodies.push(&payload.record);
            }
        }
        message_bodies.sort_by(|left, right| {
            (body_order(&left.body_type), &left.body_key)
                .cmp(&(body_order(&right.body_type), &right.body_key))
        });
        let body_content = message_bodies.iter().map(|body| {
            let payload = body_payloads.get(body.body_key.as_str()).copied()
                .filter(|payload| payload.record.message_key == body.message_key);
            body_content(body, payload)
        }).collect::<Vec<_>>();

        let mut message_recipients = recipients_by_message.get(key).cloned().unwrap_or_default();
        message_recipients.sort_by(|left, right| {
            (left.ordinal, &left.recipient_type, &left.recipient_key)
                .cmp(&(right.ordinal, &right.recipient_type, &right.recipient_key))
        });
        let mut message_attachments = attachments_by_message.get(key).cloned().unwrap_or_default();
        message_attachments.sort_by(|left, right| {
            (left.ordinal, &left.attachment_key).cmp(&(right.ordinal, &right.attachment_key))
        });
        let reconstructible = body_content.iter().any(|body| {
            body.payload_present && matches!(body.record.body_type.as_str(), "text" | "html" | "rtf")
        });
        EmailContentRecord {
            record_type: "email_content".to_string(),
            schema_version: 1,
            message: message.clone(),
            header_projection: headers_by_message.get(key).map(|header| (*header).clone()),
            recipients: message_recipients.into_iter().cloned().collect(),
            bodies: body_content,
            attachment_ids: message_attachments.iter().map(|record| record.attachment_key.clone()).collect(),
            attachments: message_attachments.into_iter().map(|record| EmailAttachmentReference { record: record.clone() }).collect(),
            reconstruction_status: if reconstructible {
                "reconstructible_body_available".to_string()
            } else {
                "body_payload_unavailable".to_string()
            },
        }
    })
}

fn group_by<'a, T>(records: &'a [T], key: impl Fn(&'a T) -> &'a str) -> BTreeMap<&'a str, Vec<&'a T>> {
    let mut groups = BTreeMap::<&str, Vec<&T>>::new();
    for record in records {
        groups.entry(key(record)).or_default().push(record);
    }
    groups
}

fn body_content(body: &BodyRecord, payload: Option<&BodyPayload>) -> EmailBodyContent {
    let (raw_bytes_base64, decoded_bytes_base64, text, rendered_html) = match payload {
        Some(payload) => {
            let (decoded_bytes_base64, rendered_html) = if payload.record.body_type == "rtf" {
                let validation = crate::pst::rtf::validate(&payload.bytes);
                (
                    validation.decoded.as_deref().map(base64_encode),
                    validation.recovered_html,
                )
            } else {
                (None, None)
            };
            (
                Some(base64_encode(&payload.bytes)),
                decoded_bytes_base64,
                std::str::from_utf8(&payload.bytes).ok().map(str::to_owned),
                rendered_html,
            )
        }
        None => (None, None, None, None),
    };
    EmailBodyContent {
        record: body.clone(),
        payload_present: payload.is_some(),
        raw_bytes_base64,
        decoded_bytes_base64,
        text,
        rendered_html,
    }
}

fn body_order(body_type: &str) -> u8 {
    match body_type {
        "text" => 0,
        "html" => 1,
        "rtf" => 2,
        "encrypted_html" => 3,
        "encrypted" => 4,
        _ => 10,
    }
}

/// Reconstruct an EML from one `email_content.jsonl` row and separately
/// retrieved attachment payloads. Attachment payloads are checked against the
/// row's stable IDs and metadata before they are admitted to the MIME output.
pub fn reconstruct_email(
    content: &EmailContentRecord,
    attachment_payloads: &[AttachmentPayload],
) -> Result<Vec<u8>, String> {
    let mut body_records = Vec::new();
    let mut body_payloads = Vec::new();
    for body in &content.bodies {
        body_records.push(body.record.clone());
        let Some(encoded) = body.raw_bytes_base64.as_deref() else {
            continue;
        };
        let bytes = base64_decode(encoded)?;
        if bytes.len() as u64 != body.record.size_bytes || sha256_hex(&bytes) != body.record.sha256
        {
            return Err(format!(
                "body payload integrity failed: {}",
                body.record.body_key
            ));
        }
        body_payloads.push(BodyPayload {
            record: body.record.clone(),
            bytes,
        });
    }

    let attachments = content
        .attachments
        .iter()
        .map(|attachment| attachment.record.clone())
        .collect::<Vec<_>>();
    let allowed_ids = content
        .attachment_ids
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    for payload in attachment_payloads {
        if !allowed_ids.contains(&payload.record.attachment_key) {
            return Err(format!(
                "attachment payload is not declared by email content: {}",
                payload.record.attachment_key
            ));
        }
        let Some(record) = attachments
            .iter()
            .find(|record| record.attachment_key == payload.record.attachment_key)
        else {
            return Err(format!(
                "attachment payload is not declared by email content: {}",
                payload.record.attachment_key
            ));
        };
        if payload.record.message_key != content.message.message_key
            || payload.bytes.len() as u64 != record.size_bytes
            || sha256_hex(&payload.bytes) != record.sha256
        {
            return Err(format!(
                "attachment payload integrity failed: {}",
                payload.record.attachment_key
            ));
        }
    }

    let filtered_payloads = attachment_payloads
        .iter()
        .filter(|payload| allowed_ids.contains(&payload.record.attachment_key))
        .cloned()
        .collect::<Vec<_>>();
    let headers = content
        .header_projection
        .clone()
        .into_iter()
        .collect::<Vec<_>>();

    crate::output::mailbox::serialize_message_eml(
        &content.message,
        &headers,
        &content.recipients,
        &body_records,
        &body_payloads,
        &attachments,
        &filtered_payloads,
        &[],
        false,
    )
    .map_err(|error| format!("email reconstruction failed: {}", error.status))
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        output.push(TABLE[(first >> 2) as usize] as char);
        output.push(TABLE[(((first & 0x03) << 4) | (second >> 4)) as usize] as char);
        output.push(if chunk.len() > 1 {
            TABLE[(((second & 0x0f) << 2) | (third >> 6)) as usize] as char
        } else {
            '='
        });
        output.push(if chunk.len() > 2 {
            TABLE[(third & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    output
}

fn base64_decode(value: &str) -> Result<Vec<u8>, String> {
    let bytes = value.as_bytes();
    if !bytes.len().is_multiple_of(4) {
        return Err("invalid base64 length".to_string());
    }
    let mut output = Vec::with_capacity(bytes.len() / 4 * 3);
    for chunk in bytes.chunks(4) {
        let first = base64_value(chunk[0]).ok_or_else(|| "invalid base64 character".to_string())?;
        let second =
            base64_value(chunk[1]).ok_or_else(|| "invalid base64 character".to_string())?;
        let third = if chunk[2] == b'=' {
            0
        } else {
            base64_value(chunk[2]).ok_or_else(|| "invalid base64 character".to_string())?
        };
        let fourth = if chunk[3] == b'=' {
            0
        } else {
            base64_value(chunk[3]).ok_or_else(|| "invalid base64 character".to_string())?
        };
        output.push((first << 2) | (second >> 4));
        if chunk[2] != b'=' {
            output.push((second << 4) | (third >> 2));
        }
        if chunk[3] != b'=' {
            output.push((third << 6) | fourth);
        }
    }
    Ok(output)
}

fn base64_value(value: u8) -> Option<u8> {
    match value {
        b'A'..=b'Z' => Some(value - b'A'),
        b'a'..=b'z' => Some(value - b'a' + 26),
        b'0'..=b'9' => Some(value - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::{base64_decode, base64_encode};

    #[test]
    fn base64_round_trips_empty_and_binary_content() {
        for bytes in [b"".as_slice(), b"hello", &[0, 1, 2, 253, 254, 255]] {
            assert_eq!(base64_decode(&base64_encode(bytes)).unwrap(), bytes);
        }
    }
}
