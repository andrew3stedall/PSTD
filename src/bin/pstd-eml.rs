use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use pstd::engine::metadata::extract_metadata;
use pstd::output::metadata::{MessageRecord, RecipientRecord};
use pstd::pst::messages::BodyPayload;

fn main() {
    if let Err(error) = run() {
        eprintln!("pstd-eml: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args_os().skip(1);
    let input = args
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "usage: pstd-eml <input.pst> <output-dir>".to_string())?;
    let output = args
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "usage: pstd-eml <input.pst> <output-dir>".to_string())?;
    if args.next().is_some() {
        return Err("usage: pstd-eml <input.pst> <output-dir>".to_string());
    }

    fs::create_dir_all(&output).map_err(|error| error.to_string())?;
    let input_display = input.display().to_string();
    let run_id = pstd::output::ids::run_id(&input_display);
    let pst_id = pstd::output::ids::pst_id(&input_display);
    let metadata = extract_metadata(&input_display, &run_id, &pst_id)
        .map_err(|error| format!("metadata extraction failed: {error}"))?;

    let recipients = recipients_by_message(&metadata.recipients);
    let bodies = text_bodies_by_message(&metadata.body_payloads);
    let mut emitted = 0usize;

    for message in &metadata.messages {
        let Some(body) = bodies.get(&message.message_key) else {
            continue;
        };
        let message_recipients = recipients
            .get(&message.message_key)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let Some(eml) = build_eml(message, message_recipients, body) else {
            continue;
        };
        let path = output.join(format!("{}.eml", safe_filename(&message.message_key)));
        fs::write(&path, eml).map_err(|error| error.to_string())?;
        emitted += 1;
    }

    println!("eml_files_emitted={emitted}");
    if emitted == 0 {
        return Err("no message had the validated sender, recipients, subject, and plain-text body required for EML emission".to_string());
    }
    Ok(())
}

fn recipients_by_message(records: &[RecipientRecord]) -> BTreeMap<String, Vec<RecipientRecord>> {
    let mut grouped: BTreeMap<String, Vec<RecipientRecord>> = BTreeMap::new();
    for record in records {
        grouped
            .entry(record.message_key.clone())
            .or_default()
            .push(record.clone());
    }
    for records in grouped.values_mut() {
        records.sort_by_key(|record| record.ordinal);
    }
    grouped
}

fn text_bodies_by_message(payloads: &[BodyPayload]) -> BTreeMap<String, Vec<u8>> {
    let mut bodies = BTreeMap::new();
    for payload in payloads {
        if payload.record.body_type == "text" && !payload.bytes.is_empty() {
            bodies
                .entry(payload.record.message_key.clone())
                .or_insert_with(|| payload.bytes.clone());
        }
    }
    bodies
}

fn build_eml(message: &MessageRecord, recipients: &[RecipientRecord], body: &[u8]) -> Option<Vec<u8>> {
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

    let body = std::str::from_utf8(body).ok()?;
    let mut eml = String::new();
    push_header(&mut eml, "From", &from);
    if let Some(to) = to {
        push_header(&mut eml, "To", &to);
    }
    if let Some(cc) = cc {
        push_header(&mut eml, "Cc", &cc);
    }
    push_header(&mut eml, "Subject", &subject);
    if let Some(message_id) = message.internet_message_id.as_deref().and_then(clean_header) {
        push_header(&mut eml, "Message-ID", &message_id);
    }
    push_header(&mut eml, "MIME-Version", "1.0");
    push_header(&mut eml, "Content-Type", "text/plain; charset=utf-8");
    push_header(&mut eml, "Content-Transfer-Encoding", "8bit");
    eml.push_str("\r\n");
    eml.push_str(&normalize_crlf(body));
    if !eml.ends_with("\r\n") {
        eml.push_str("\r\n");
    }
    Some(eml.into_bytes())
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
        format!("\"{}\"", value.replace('\\', "\\\\").replace('\"', "\\\""))
    }
}

fn clean_header(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value.contains('\r') || value.contains('\n') {
        None
    } else {
        Some(value.to_string())
    }
}

fn push_header(output: &mut String, name: &str, value: &str) {
    output.push_str(name);
    output.push_str(": ");
    output.push_str(value);
    output.push_str("\r\n");
}

fn normalize_crlf(value: &str) -> String {
    value.replace("\r\n", "\n").replace('\r', "\n").replace('\n', "\r\n")
}

fn safe_filename(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message() -> MessageRecord {
        MessageRecord {
            run_id: "run".to_string(),
            pst_id: "pst".to_string(),
            folder_key: "folder".to_string(),
            message_key: "message".to_string(),
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
            transport_message_headers: None,
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

    fn recipient(ordinal: u64, role: &str, name: &str, address: &str) -> RecipientRecord {
        RecipientRecord {
            message_key: "message".to_string(),
            recipient_key: format!("recipient-{ordinal}"),
            recipient_type: role.to_string(),
            display_name: Some(name.to_string()),
            raw_address: Some(address.to_string()),
            address_type: Some("native_email_address".to_string()),
            smtp_address: None,
            resolution_status: "validated".to_string(),
            ordinal,
        }
    }

    #[test]
    fn emits_readable_crlf_message_from_validated_fields() {
        let recipients = vec![
            recipient(0, "to", "Recipient 1", "to1@domain.com"),
            recipient(1, "to", "Recipient 2", "to2@domain.com"),
            recipient(2, "cc", "Recipient 3", "cc1@domain.com"),
        ];
        let eml = build_eml(&message(), &recipients, b"Hello\nworld").unwrap();
        let eml = String::from_utf8(eml).unwrap();
        assert!(eml.contains("From: Fixture Sender <sender@example.com>\r\n"));
        assert!(eml.contains(
            "To: Recipient 1 <to1@domain.com>, Recipient 2 <to2@domain.com>\r\n"
        ));
        assert!(eml.contains("Cc: Recipient 3 <cc1@domain.com>\r\n"));
        assert!(eml.contains("Subject: Fixture subject\r\n"));
        assert!(eml.ends_with("\r\n\r\nHello\r\nworld\r\n"));
    }

    #[test]
    fn fails_closed_for_header_injection_or_missing_recipients() {
        let mut invalid = message();
        invalid.subject = Some("bad\r\nBcc: attacker@example.com".to_string());
        assert!(build_eml(&invalid, &[], b"body").is_none());
        assert!(build_eml(&message(), &[], b"body").is_none());
    }

    #[test]
    fn rejects_non_utf8_plain_text_instead_of_guessing() {
        let recipients = vec![recipient(0, "to", "Recipient", "to@domain.com")];
        assert!(build_eml(&message(), &recipients, &[0xff, 0xfe]).is_none());
    }
}
