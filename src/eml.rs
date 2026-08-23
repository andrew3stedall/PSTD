use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, FixedOffset, Utc};
use sha2::{Digest, Sha256};

use crate::output::headers::{
    clean_header_value, encode_display_name, encode_mime_parameter, encode_unstructured_value,
    normalize_content_id,
};
use crate::output::metadata::{AttachmentRecord, MessageRecord, RecipientRecord};
use crate::pst::attachments::{AttachmentPayload, ATTACH_METHOD_EMBEDDED_MESSAGE};
use crate::pst::messages::BodyPayload;

const FILETIME_UNIX_EPOCH_TICKS: u64 = 116_444_736_000_000_000;
const MIXED_BOUNDARY: &str = "pstd-mixed-3e2b1a9c";

pub fn build_inline_eml_with_attachments(
    message: &MessageRecord,
    recipients: &[RecipientRecord],
    text_bytes: &[u8],
    attachments: &[AttachmentPayload],
) -> Option<Vec<u8>> {
    let subject = clean_header(message.subject.as_deref()?)?;
    let sender_address = message
        .sender_email
        .as_deref()
        .or(message.sender_raw_address.as_deref())
        .and_then(clean_header)?;
    let sender_name = message.sender_name.as_deref().and_then(clean_header);
    let from = format_address(sender_name.as_deref(), &sender_address);
    let to = recipient_header(recipients, "to");
    let cc = recipient_header(recipients, "cc");
    if to.is_none() && cc.is_none() {
        return None;
    }
    let date = validated_message_date(message)?;
    let text = std::str::from_utf8(text_bytes).ok()?;
    if text.contains(MIXED_BOUNDARY) || !attachment_payloads_are_valid(attachments) {
        return None;
    }

    let mut eml = String::new();
    push_header(&mut eml, "From", &from);
    if let Some(to) = to {
        push_header(&mut eml, "To", &to);
    }
    if let Some(cc) = cc {
        push_header(&mut eml, "Cc", &cc);
    }
    push_header(&mut eml, "Subject", &encode_unstructured_value(&subject));
    push_header(&mut eml, "Date", &date);
    if let Some(message_id) = message
        .internet_message_id
        .as_deref()
        .and_then(clean_header)
    {
        push_header(&mut eml, "Message-ID", &message_id);
    }
    push_header(&mut eml, "MIME-Version", "1.0");

    if attachments.is_empty() {
        push_header(&mut eml, "Content-Type", "text/plain; charset=utf-8");
        push_header(&mut eml, "Content-Transfer-Encoding", "8bit");
        eml.push_str("\r\n");
        eml.push_str(&normalize_crlf(text));
        if !eml.ends_with("\r\n") {
            eml.push_str("\r\n");
        }
    } else {
        push_header(
            &mut eml,
            "Content-Type",
            &format!("multipart/mixed; boundary=\"{MIXED_BOUNDARY}\""),
        );
        eml.push_str("\r\n--");
        eml.push_str(MIXED_BOUNDARY);
        eml.push_str("\r\n");
        push_header(&mut eml, "Content-Type", "text/plain; charset=utf-8");
        push_header(&mut eml, "Content-Transfer-Encoding", "8bit");
        eml.push_str("\r\n");
        eml.push_str(&normalize_crlf(text));
        if !eml.ends_with("\r\n") {
            eml.push_str("\r\n");
        }
        for attachment in attachments {
            push_attachment_part(&mut eml, attachment)?;
        }
        eml.push_str("--");
        eml.push_str(MIXED_BOUNDARY);
        eml.push_str("--\r\n");
    }
    Some(eml.into_bytes())
}

pub fn build_plain_text_eml(
    message: &MessageRecord,
    recipients: &[RecipientRecord],
    text_bytes: &[u8],
) -> Option<Vec<u8>> {
    build_inline_eml_with_attachments(message, recipients, text_bytes, &[])
}

pub fn materialize_embedded_message_payloads(
    attachments: &mut [AttachmentRecord],
    payloads: &mut Vec<AttachmentPayload>,
    messages: &[MessageRecord],
    recipients: &[RecipientRecord],
    bodies: &[BodyPayload],
) -> usize {
    let mut message_counts = BTreeMap::<&str, usize>::new();
    for message in messages {
        *message_counts.entry(&message.message_key).or_default() += 1;
    }
    let mut body_counts = BTreeMap::<&str, usize>::new();
    for body in bodies.iter().filter(|body| body.record.body_type == "text") {
        *body_counts.entry(&body.record.message_key).or_default() += 1;
    }
    let duplicate_attachment_keys = duplicate_values(
        attachments
            .iter()
            .map(|attachment| attachment.attachment_key.clone()),
    );
    let duplicate_embedded_keys = duplicate_values(attachments.iter().filter_map(|attachment| {
        (attachment.attachment_method == Some(ATTACH_METHOD_EMBEDDED_MESSAGE))
            .then_some(attachment.embedded_message_key.clone())
            .flatten()
    }));
    let attachment_records = attachments.to_vec();
    let mut materialized = 0usize;
    for attachment in attachments.iter_mut() {
        if attachment.attachment_method != Some(ATTACH_METHOD_EMBEDDED_MESSAGE)
            || duplicate_attachment_keys.contains(attachment.attachment_key.as_str())
        {
            continue;
        }
        let Some(child_key) = attachment.embedded_message_key.as_deref() else {
            continue;
        };
        if duplicate_embedded_keys.contains(child_key)
            || message_counts.get(child_key) != Some(&1)
            || body_counts.get(child_key) != Some(&1)
        {
            continue;
        }
        let Some(message) = messages
            .iter()
            .find(|message| message.message_key == child_key)
        else {
            continue;
        };
        let child_recipients = recipients
            .iter()
            .filter(|recipient| recipient.message_key == child_key)
            .cloned()
            .collect::<Vec<_>>();
        let Some(body) = bodies
            .iter()
            .find(|body| body.record.message_key == child_key && body.record.body_type == "text")
        else {
            continue;
        };
        let child_attachment_records = attachment_records
            .iter()
            .filter(|candidate| candidate.message_key == child_key)
            .collect::<Vec<_>>();
        if child_attachment_records
            .iter()
            .any(|candidate| candidate.attachment_method == Some(ATTACH_METHOD_EMBEDDED_MESSAGE))
        {
            continue;
        }
        let mut child_payloads = child_attachment_records
            .iter()
            .filter_map(|candidate| {
                payloads
                    .iter()
                    .find(|payload| payload.record.attachment_key == candidate.attachment_key)
                    .cloned()
            })
            .collect::<Vec<_>>();
        if child_payloads.len() != child_attachment_records.len() {
            continue;
        }
        child_payloads.sort_by_key(|payload| attachment_order_key(&payload.record));
        let Some(bytes) = build_inline_eml_with_attachments(
            message,
            &child_recipients,
            &body.bytes,
            &child_payloads,
        ) else {
            continue;
        };
        attachment.content_type = Some("message/rfc822".to_string());
        attachment.size_bytes = bytes.len() as u64;
        attachment.sha256 = sha256_hex(&bytes);
        attachment.size_status = match attachment.declared_size_bytes {
            Some(declared) if declared == attachment.size_bytes => "size_matched".to_string(),
            Some(_) => "size_mismatch".to_string(),
            None => "declared_size_absent".to_string(),
        };
        attachment.extraction_status = "extracted_embedded_message_eml".to_string();
        payloads.push(AttachmentPayload {
            record: attachment.clone(),
            bytes,
        });
        materialized += 1;
    }
    materialized
}

fn duplicate_values(values: impl Iterator<Item = String>) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut duplicates = BTreeSet::new();
    for value in values {
        if !seen.insert(value.clone()) {
            duplicates.insert(value);
        }
    }
    duplicates
}

fn attachment_order_key(record: &AttachmentRecord) -> (bool, u64, u64, u64, String) {
    (
        record.mime_sequence.is_none(),
        record.mime_sequence.unwrap_or(u64::MAX),
        record.rendering_position.unwrap_or(u64::MAX),
        record.ordinal,
        record.attachment_key.clone(),
    )
}

fn attachment_payloads_are_valid(attachments: &[AttachmentPayload]) -> bool {
    let mut attachment_keys = BTreeSet::new();
    let mut ordinals = BTreeSet::new();
    attachments.iter().all(|attachment| {
        attachment.record.size_bytes == attachment.bytes.len() as u64
            && attachment.record.sha256 == sha256_hex(&attachment.bytes)
            && attachment_keys.insert(attachment.record.attachment_key.clone())
            && ordinals.insert(attachment.record.ordinal)
    })
}

fn push_attachment_part(output: &mut String, attachment: &AttachmentPayload) -> Option<()> {
    let filename = clean_header(&attachment.record.filename_safe)?;
    let content_type = attachment_content_type(attachment);
    let disposition = if attachment.record.is_inline {
        "inline"
    } else {
        "attachment"
    };

    output.push_str("--");
    output.push_str(MIXED_BOUNDARY);
    output.push_str("\r\n");
    push_header(
        output,
        "Content-Type",
        &format!(
            "{content_type}; {}",
            encode_mime_parameter("name", &filename)
        ),
    );
    push_header(output, "Content-Transfer-Encoding", "base64");
    push_header(
        output,
        "Content-Disposition",
        &format!(
            "{disposition}; {}",
            encode_mime_parameter("filename", &filename)
        ),
    );
    if let Some(content_id) = attachment
        .record
        .content_id
        .as_deref()
        .and_then(normalize_content_id)
    {
        push_header(output, "Content-ID", &content_id);
    }
    output.push_str("\r\n");
    output.push_str(&base64_lines(&attachment.bytes));
    Some(())
}

fn attachment_content_type(attachment: &AttachmentPayload) -> String {
    attachment
        .record
        .content_type
        .as_deref()
        .and_then(clean_header)
        .or_else(|| match attachment.record.extension.as_deref() {
            _ if attachment.record.attachment_method == Some(5) => {
                Some("message/rfc822".to_string())
            }
            Some("docx") => Some(
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                    .to_string(),
            ),
            _ => None,
        })
        .unwrap_or_else(|| "application/octet-stream".to_string())
}

fn base64_lines(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        encoded.push(ALPHABET[(first >> 2) as usize] as char);
        encoded.push(ALPHABET[(((first & 0x03) << 4) | (second >> 4)) as usize] as char);
        encoded.push(if chunk.len() > 1 {
            ALPHABET[(((second & 0x0f) << 2) | (third >> 6)) as usize] as char
        } else {
            '='
        });
        encoded.push(if chunk.len() > 2 {
            ALPHABET[(third & 0x3f) as usize] as char
        } else {
            '='
        });
    }

    let mut lines = String::with_capacity(encoded.len() + encoded.len() / 76 * 2 + 2);
    for line in encoded.as_bytes().chunks(76) {
        lines.push_str(std::str::from_utf8(line).expect("base64 is ASCII"));
        lines.push_str("\r\n");
    }
    lines
}

fn validated_message_date(message: &MessageRecord) -> Option<String> {
    validated_transport_date(message)
        .or_else(|| message.sent_at.as_deref().and_then(validated_filetime_date))
        .or_else(|| {
            message
                .received_at
                .as_deref()
                .and_then(validated_filetime_date)
        })
}

fn validated_filetime_date(value: &str) -> Option<String> {
    let ticks = value.strip_prefix("filetime:")?.parse::<u64>().ok()?;
    let unix_ticks = ticks.checked_sub(FILETIME_UNIX_EPOCH_TICKS)?;
    let seconds = i64::try_from(unix_ticks / 10_000_000).ok()?;
    let nanoseconds = u32::try_from((unix_ticks % 10_000_000) * 100).ok()?;
    let parsed = DateTime::<Utc>::from_timestamp(seconds, nanoseconds)?;
    Some(parsed.format("%a, %d %b %Y %H:%M:%S +0000").to_string())
}

fn validated_transport_date(message: &MessageRecord) -> Option<String> {
    let headers = message.transport_message_headers.as_deref()?;
    let mut dates = headers.lines().filter_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case("Date")
            .then(|| clean_header(value))
            .flatten()
    });
    let value = dates.next()?;
    if dates.next().is_some() {
        return None;
    }
    let parsed: DateTime<FixedOffset> = DateTime::parse_from_rfc2822(&value).ok()?;
    Some(parsed.format("%a, %d %b %Y %H:%M:%S %z").to_string())
}

fn recipient_header(records: &[RecipientRecord], role: &str) -> Option<String> {
    let values = records
        .iter()
        .filter(|record| record.recipient_type == role)
        .filter_map(|record| {
            let address = record
                .smtp_address
                .as_deref()
                .or(record.raw_address.as_deref())
                .and_then(clean_header)?;
            let name = record.display_name.as_deref().and_then(clean_header);
            Some(format_address(name.as_deref(), &address))
        })
        .collect::<Vec<_>>();
    (!values.is_empty()).then(|| values.join(", "))
}

fn format_address(name: Option<&str>, address: &str) -> String {
    match name.filter(|name| !name.is_empty()) {
        Some(name) => format!("{} <{}>", encode_display_name(name), address),
        None => address.to_string(),
    }
}

fn clean_header(value: &str) -> Option<String> {
    clean_header_value(value)
}

fn push_header(output: &mut String, name: &str, value: &str) {
    output.push_str(name);
    output.push_str(": ");
    output.push_str(value);
    output.push_str("\r\n");
}

fn normalize_crlf(value: &str) -> String {
    value
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\r\n")
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}
