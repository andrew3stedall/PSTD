use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, FixedOffset, Utc};
use sha2::{Digest, Sha256};

use crate::output::metadata::{AttachmentRecord, MessageRecord, RecipientRecord};
use crate::pst::attachments::{AttachmentPayload, ATTACH_METHOD_EMBEDDED_MESSAGE};
use crate::pst::messages::BodyPayload;

const FILETIME_UNIX_EPOCH_TICKS: u64 = 116_444_736_000_000_000;

pub fn build_plain_text_eml(
    message: &MessageRecord,
    recipients: &[RecipientRecord],
    text_bytes: &[u8],
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

    let mut eml = String::new();
    push_header(&mut eml, "From", &from);
    if let Some(to) = to {
        push_header(&mut eml, "To", &to);
    }
    if let Some(cc) = cc {
        push_header(&mut eml, "Cc", &cc);
    }
    push_header(&mut eml, "Subject", &subject);
    push_header(&mut eml, "Date", &date);
    if let Some(message_id) = message
        .internet_message_id
        .as_deref()
        .and_then(clean_header)
    {
        push_header(&mut eml, "Message-ID", &message_id);
    }
    push_header(&mut eml, "MIME-Version", "1.0");
    push_header(&mut eml, "Content-Type", "text/plain; charset=utf-8");
    push_header(&mut eml, "Content-Transfer-Encoding", "8bit");
    eml.push_str("\r\n");
    eml.push_str(&normalize_crlf(text));
    if !eml.ends_with("\r\n") {
        eml.push_str("\r\n");
    }
    Some(eml.into_bytes())
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
    let child_messages_with_attachments = attachments
        .iter()
        .map(|attachment| attachment.message_key.clone())
        .collect::<BTreeSet<_>>();

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
            || child_messages_with_attachments.contains(child_key)
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
        let Some(bytes) = build_plain_text_eml(message, &child_recipients, &body.bytes) else {
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
        Some(name) => format!("{} <{}>", quote_display_name(name), address),
        None => address.to_string(),
    }
}

fn quote_display_name(value: &str) -> String {
    if value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || " ._-".contains(character))
    {
        value.to_string()
    } else {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

fn clean_header(value: &str) -> Option<String> {
    if value.contains('\r') || value.contains('\n') {
        return None;
    }
    let cleaned = value
        .chars()
        .filter(|character| !character.is_control() || *character == '\t')
        .collect::<String>();
    let cleaned = cleaned.trim();
    (!cleaned.is_empty()).then(|| cleaned.to_string())
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
