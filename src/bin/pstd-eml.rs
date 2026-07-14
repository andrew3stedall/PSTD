use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, FixedOffset};
use pstd::engine::metadata::extract_metadata;
use pstd::output::metadata::{MessageRecord, RecipientRecord};
use pstd::pst::messages::BodyPayload;

const LZFU_MAGIC: u32 = 0x7546_5a4c;
const MELA_MAGIC: u32 = 0x414c_454d;
const INITIAL_DICTIONARY: &[u8] = b"{\\rtf1\\ansi\\mac\\deff0\\deftab720{\\fonttbl;}{\\f0\\fnil \\froman \\fswiss \\fmodern \\fscript \\fdecor MS Sans SerifSymbolArialTimes New RomanCourier{\\colortbl\\red0\\green0\\blue0\r\n\\par \\pard\\plain\\f0\\fs20\\b\\i\\u\\tab\\tx";
const ALTERNATIVE_BOUNDARY: &str = "pstd-alternative-7f6a8d2b";

fn main() {
    if let Err(error) = run() {
        eprintln!("pstd-eml: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args_os().skip(1);
    let input = args.next().map(PathBuf::from).ok_or_else(usage)?;
    let output = args.next().map(PathBuf::from).ok_or_else(usage)?;
    if args.next().is_some() {
        return Err(usage());
    }

    fs::create_dir_all(&output).map_err(|error| error.to_string())?;
    let input_display = input.display().to_string();
    let run_id = pstd::output::ids::run_id(&input_display);
    let pst_id = pstd::output::ids::pst_id(&input_display);
    let metadata = extract_metadata(&input_display, &run_id, &pst_id)
        .map_err(|error| format!("metadata extraction failed: {error}"))?;

    let recipients = recipients_by_message(&metadata.recipients);
    let bodies = bodies_by_message(&metadata.body_payloads);
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
        fs::write(path, eml).map_err(|error| error.to_string())?;
        emitted += 1;
    }

    println!("eml_files_emitted={emitted}");
    if emitted == 0 {
        return Err("no message had validated sender, recipients, subject, plain-text body, and readable RTF required for multipart EML emission".to_string());
    }
    Ok(())
}

fn usage() -> String {
    "usage: pstd-eml <input.pst> <output-dir>".to_string()
}

#[derive(Default)]
struct MessageBodies {
    text: Option<Vec<u8>>,
    rtf: Option<Vec<u8>>,
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

fn bodies_by_message(payloads: &[BodyPayload]) -> BTreeMap<String, MessageBodies> {
    let mut bodies: BTreeMap<String, MessageBodies> = BTreeMap::new();
    for payload in payloads {
        let entry = bodies
            .entry(payload.record.message_key.clone())
            .or_default();
        match payload.record.body_type.as_str() {
            "text" if !payload.bytes.is_empty() && entry.text.is_none() => {
                entry.text = Some(payload.bytes.clone());
            }
            "rtf" if entry.rtf.is_none() => {
                entry.rtf = validated_rtf(&payload.bytes);
            }
            _ => {}
        }
    }
    bodies.retain(|_, body| body.text.is_some() && body.rtf.is_some());
    bodies
}

fn build_eml(
    message: &MessageRecord,
    recipients: &[RecipientRecord],
    bodies: &MessageBodies,
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

    let text = std::str::from_utf8(bodies.text.as_deref()?).ok()?;
    let rtf = std::str::from_utf8(bodies.rtf.as_deref()?).ok()?;
    if text.contains(ALTERNATIVE_BOUNDARY) || rtf.contains(ALTERNATIVE_BOUNDARY) {
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
    push_header(&mut eml, "Subject", &subject);
    if let Some(date) = validated_transport_date(message) {
        push_header(&mut eml, "Date", &date);
    }
    if let Some(message_id) = message
        .internet_message_id
        .as_deref()
        .and_then(clean_header)
    {
        push_header(&mut eml, "Message-ID", &message_id);
    }
    push_header(&mut eml, "MIME-Version", "1.0");
    push_header(
        &mut eml,
        "Content-Type",
        &format!("multipart/alternative; boundary=\"{ALTERNATIVE_BOUNDARY}\""),
    );
    eml.push_str("\r\n");
    push_part(&mut eml, "text/plain; charset=utf-8", &normalize_crlf(text));
    push_part(&mut eml, "text/rtf; charset=utf-8", &normalize_crlf(rtf));
    eml.push_str("--");
    eml.push_str(ALTERNATIVE_BOUNDARY);
    eml.push_str("--\r\n");
    Some(eml.into_bytes())
}

fn push_part(output: &mut String, content_type: &str, body: &str) {
    output.push_str("--");
    output.push_str(ALTERNATIVE_BOUNDARY);
    output.push_str("\r\n");
    push_header(output, "Content-Type", content_type);
    push_header(output, "Content-Transfer-Encoding", "8bit");
    output.push_str("\r\n");
    output.push_str(body);
    if !output.ends_with("\r\n") {
        output.push_str("\r\n");
    }
}

fn validated_rtf(bytes: &[u8]) -> Option<Vec<u8>> {
    if bytes.starts_with(b"{\\rtf") {
        return Some(bytes.to_vec());
    }
    let decoded = decompress_rtf(bytes)?;
    decoded.starts_with(b"{\\rtf").then_some(decoded)
}

fn decompress_rtf(input: &[u8]) -> Option<Vec<u8>> {
    if input.len() < 16 {
        return None;
    }
    let compressed_size = read_u32(input, 0)? as usize;
    let raw_size = read_u32(input, 4)? as usize;
    let magic = read_u32(input, 8)?;
    let expected_crc = read_u32(input, 12)?;
    if compressed_size.checked_add(4)? != input.len() {
        return None;
    }
    let payload = &input[16..];
    match magic {
        MELA_MAGIC => {
            if expected_crc != 0
                || compressed_size != raw_size
                || payload.len().checked_add(12)? != raw_size
            {
                return None;
            }
            Some(payload.to_vec())
        }
        LZFU_MAGIC => {
            if crc32(payload) != expected_crc {
                return None;
            }
            let decoded = decompress_lzfu(payload, raw_size)?;
            (decoded.len() == raw_size).then_some(decoded)
        }
        _ => None,
    }
}

fn decompress_lzfu(input: &[u8], raw_size: usize) -> Option<Vec<u8>> {
    let mut dictionary = [0u8; 4096];
    dictionary[..INITIAL_DICTIONARY.len()].copy_from_slice(INITIAL_DICTIONARY);
    let mut dictionary_position = INITIAL_DICTIONARY.len();
    let mut output = Vec::with_capacity(raw_size);
    let mut input_position = 0usize;
    while output.len() < raw_size {
        let flags = *input.get(input_position)?;
        input_position += 1;
        for bit in 0..8 {
            if output.len() == raw_size {
                break;
            }
            if flags & (1 << bit) == 0 {
                let value = *input.get(input_position)?;
                input_position += 1;
                output.push(value);
                dictionary[dictionary_position & 0x0fff] = value;
                dictionary_position = (dictionary_position + 1) & 0x0fff;
            } else {
                let first = *input.get(input_position)? as usize;
                let second = *input.get(input_position + 1)? as usize;
                input_position += 2;
                let mut reference = (first << 4) | (second >> 4);
                let length = (second & 0x0f) + 2;
                for _ in 0..length {
                    if output.len() == raw_size {
                        break;
                    }
                    let value = dictionary[reference & 0x0fff];
                    reference = (reference + 1) & 0x0fff;
                    output.push(value);
                    dictionary[dictionary_position & 0x0fff] = value;
                    dictionary_position = (dictionary_position + 1) & 0x0fff;
                }
            }
        }
    }
    Some(output)
}

fn read_u32(input: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        input.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320 & (0u32.wrapping_sub(crc & 1)));
        }
    }
    !crc
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
    use pstd::pst::messages::body_payload;

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
            transport_message_headers: Some(
                "Date: 19 Aug 2015 11:07:26 +0000\r\n".to_string(),
            ),
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

    fn recipient(ordinal: u64, role: &str) -> RecipientRecord {
        RecipientRecord {
            message_key: "message".to_string(),
            recipient_key: format!("recipient-{ordinal}"),
            recipient_type: role.to_string(),
            display_name: Some(format!("Recipient {ordinal}")),
            raw_address: Some(format!("r{ordinal}@example.com")),
            address_type: Some("native_email_address".to_string()),
            smtp_address: None,
            resolution_status: "validated".to_string(),
            ordinal,
        }
    }

    #[test]
    fn emits_multipart_plain_and_rtf_message() {
        let bodies = MessageBodies {
            text: Some(b"Hello\nworld".to_vec()),
            rtf: Some(b"{\\rtf1\\ansi Rich body}".to_vec()),
        };
        let eml = build_eml(
            &message(),
            &[recipient(0, "to"), recipient(1, "cc")],
            &bodies,
        )
        .unwrap();
        let eml = String::from_utf8(eml).unwrap();
        assert!(eml.contains(
            "Content-Type: multipart/alternative; boundary=\"pstd-alternative-7f6a8d2b\"\r\n"
        ));
        assert!(eml.contains("Content-Type: text/plain; charset=utf-8\r\n"));
        assert!(eml.contains("Content-Type: text/rtf; charset=utf-8\r\n"));
        assert!(eml.contains("Hello\r\nworld"));
        assert!(eml.contains("{\\rtf1\\ansi Rich body}"));
        assert!(eml.ends_with("--pstd-alternative-7f6a8d2b--\r\n"));
    }

    #[test]
    fn groups_only_messages_with_both_valid_body_representations() {
        let payloads = vec![
            body_payload("message", "text", b"plain".to_vec(), None),
            body_payload("message", "rtf", b"{\\rtf1 rich}".to_vec(), None),
            body_payload("other", "text", b"plain".to_vec(), None),
        ];
        let grouped = bodies_by_message(&payloads);
        assert!(grouped.contains_key("message"));
        assert!(!grouped.contains_key("other"));
    }

    #[test]
    fn fails_closed_for_missing_or_invalid_rtf_and_boundary_collision() {
        let recipients = vec![recipient(0, "to")];
        let missing = MessageBodies {
            text: Some(b"plain".to_vec()),
            rtf: None,
        };
        assert!(build_eml(&message(), &recipients, &missing).is_none());
        let collision = MessageBodies {
            text: Some(ALTERNATIVE_BOUNDARY.as_bytes().to_vec()),
            rtf: Some(b"{\\rtf1 rich}".to_vec()),
        };
        assert!(build_eml(&message(), &recipients, &collision).is_none());
    }
}
