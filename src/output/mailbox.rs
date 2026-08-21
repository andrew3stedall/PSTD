use std::collections::{BTreeMap, BTreeSet};

use sha2::{Digest, Sha256};

use crate::config::OutputProfile;
use crate::output::ids;
use crate::output::metadata::{
    AttachmentRecord, HeaderProjectionRecord, MessageRecord, RecipientRecord,
};
use crate::output::paths::sanitize_segment;
use crate::pst::attachments::AttachmentPayload;
use crate::pst::item_routing::classify_message_class;
use crate::pst::messages::BodyPayload;

use crate::output::metadata::{BodyRecord, FolderRecord};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MailboxArtifactSummary {
    pub path: String,
    pub message_key: Option<String>,
    pub folder_path: String,
    pub output_kind: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct MailboxArtifact {
    pub summary: MailboxArtifactSummary,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MailboxProfileStatus {
    pub profile: String,
    pub status: String,
    pub output_root: String,
    pub message_count: usize,
    pub emitted_message_count: usize,
    pub artifact_count: usize,
    pub skipped_count: usize,
    pub unavailable_count: usize,
    pub path_policy: String,
    pub publication_policy: String,
    pub artifacts: Vec<MailboxArtifactSummary>,
    pub decisions: Vec<MailboxDecision>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MailboxDecision {
    pub message_key: String,
    pub folder_path: String,
    pub status: String,
    pub source_class: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MailboxRender {
    pub status: MailboxProfileStatus,
    pub artifacts: Vec<MailboxArtifact>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MailMode {
    Mbox,
    RecursiveMbox,
    Mh,
    Eml,
    Separate,
}

impl MailMode {
    fn from_profile(profile: OutputProfile) -> Option<Self> {
        match profile {
            OutputProfile::Mbox => Some(Self::Mbox),
            OutputProfile::RecursiveMbox => Some(Self::RecursiveMbox),
            OutputProfile::Mh => Some(Self::Mh),
            OutputProfile::Eml => Some(Self::Eml),
            OutputProfile::Separate => Some(Self::Separate),
            _ => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Mbox => "mbox",
            Self::RecursiveMbox => "recursive_mbox",
            Self::Mh => "mh",
            Self::Eml => "eml",
            Self::Separate => "separate",
        }
    }

    fn output_root(self) -> &'static str {
        match self {
            Self::Mbox => "outputs/mbox",
            Self::RecursiveMbox => "outputs/recursive_mbox",
            Self::Mh => "outputs/mh",
            Self::Eml => "outputs/eml",
            Self::Separate => "outputs/separate",
        }
    }
}

#[derive(Debug)]
struct RenderError {
    status: &'static str,
}

pub fn render_profile(
    profile: OutputProfile,
    folders: &[FolderRecord],
    messages: &[MessageRecord],
    headers: &[HeaderProjectionRecord],
    recipients: &[RecipientRecord],
    bodies: &[BodyRecord],
    body_payloads: &[BodyPayload],
    attachments: &[AttachmentRecord],
    attachment_payloads: &[AttachmentPayload],
) -> Option<MailboxRender> {
    let mode = MailMode::from_profile(profile)?;
    let ordered_messages = ordered_mail_messages(messages);
    let message_count = ordered_messages.len();
    let mut decisions = Vec::new();
    let mut eligible = Vec::new();
    let mut skipped_count = 0usize;
    let mut unavailable_count = 0usize;

    for message in ordered_messages {
        let classification = classify_message_class(message.message_class.as_deref());
        let is_mail = classification.kind.is_some_and(|kind| {
            matches!(
                kind,
                crate::output::metadata::ItemKind::Note
                    | crate::output::metadata::ItemKind::Schedule
                    | crate::output::metadata::ItemKind::Report
            )
        });
        if !is_mail {
            let status = if classification.kind.is_none() {
                "unavailable_missing_item_class"
            } else {
                "skipped_non_mail_item"
            };
            if status.starts_with("unavailable") {
                unavailable_count += 1;
            } else {
                skipped_count += 1;
            }
            decisions.push(MailboxDecision {
                message_key: message.message_key.clone(),
                folder_path: message.folder_path.clone(),
                status: status.to_string(),
                source_class: message.message_class.clone(),
            });
            continue;
        }
        eligible.push(message);
    }

    let folder_paths = folder_path_map(folders, &eligible);
    let mut artifacts = Vec::new();
    let mut emitted_message_count = 0usize;

    let mut groups = BTreeMap::<String, Vec<&MessageRecord>>::new();
    for message in eligible {
        let output_folder = folder_paths
            .get(&(message.folder_key.clone(), message.folder_path.clone()))
            .cloned()
            .unwrap_or_else(|| safe_folder_path(&message.folder_path));
        groups.entry(output_folder).or_default().push(message);
    }

    for group in groups.values_mut() {
        group.sort_by(|left, right| left.message_key.cmp(&right.message_key));
    }

    for (folder, group) in &groups {
        match mode {
            MailMode::Mbox | MailMode::RecursiveMbox => {
                let mut output = Vec::new();
                for message in group {
                    match serialize_message_eml(
                        message,
                        headers,
                        recipients,
                        bodies,
                        body_payloads,
                        attachments,
                        attachment_payloads,
                    ) {
                        Ok(eml) => {
                            output.extend_from_slice(&mbox_separator(message).into_bytes());
                            output.extend_from_slice(&mboxrd_escape(&eml));
                            emitted_message_count += 1;
                            decisions.push(MailboxDecision {
                                message_key: message.message_key.clone(),
                                folder_path: message.folder_path.clone(),
                                status: "emitted_mbox_message".to_string(),
                                source_class: message.message_class.clone(),
                            });
                        }
                        Err(error) => {
                            unavailable_count += 1;
                            decisions.push(MailboxDecision {
                                message_key: message.message_key.clone(),
                                folder_path: message.folder_path.clone(),
                                status: error.status.to_string(),
                                source_class: message.message_class.clone(),
                            });
                        }
                    }
                }
                if !output.is_empty() {
                    let path = if mode == MailMode::Mbox {
                        format!("{}/{}/email.mbox", mode.output_root(), folder)
                    } else {
                        format!("{}/{}/mbox", mode.output_root(), folder)
                    };
                    artifacts.push(artifact(
                        path,
                        None,
                        group[0].folder_path.clone(),
                        mode.name(),
                        output,
                        "mailbox_stream_emitted",
                    ));
                }
            }
            MailMode::Mh | MailMode::Eml | MailMode::Separate => {
                for (ordinal, message) in group.iter().enumerate() {
                    match serialize_message_eml(
                        message,
                        headers,
                        recipients,
                        bodies,
                        body_payloads,
                        attachments,
                        attachment_payloads,
                    ) {
                        Ok(eml) => {
                            let bytes = eml;
                            let filename = match mode {
                                MailMode::Mh | MailMode::Separate => {
                                    format!("{}", ordinal + 1)
                                }
                                MailMode::Eml => format!("{}.eml", ordinal + 1),
                                _ => unreachable!(),
                            };
                            let path = format!("{}/{}/{}", mode.output_root(), folder, filename);
                            artifacts.push(artifact(
                                path,
                                Some(message.message_key.clone()),
                                message.folder_path.clone(),
                                mode.name(),
                                bytes,
                                "message_file_emitted",
                            ));
                            emitted_message_count += 1;
                            decisions.push(MailboxDecision {
                                message_key: message.message_key.clone(),
                                folder_path: message.folder_path.clone(),
                                status: "emitted_message_file".to_string(),
                                source_class: message.message_class.clone(),
                            });
                        }
                        Err(error) => {
                            unavailable_count += 1;
                            decisions.push(MailboxDecision {
                                message_key: message.message_key.clone(),
                                folder_path: message.folder_path.clone(),
                                status: error.status.to_string(),
                                source_class: message.message_class.clone(),
                            });
                        }
                    }
                }
            }
        }
    }

    decisions.sort_by(|left, right| {
        left.message_key
            .cmp(&right.message_key)
            .then(left.status.cmp(&right.status))
    });
    let status = if message_count == 0 || emitted_message_count == 0 {
        "mail_records_unavailable"
    } else if unavailable_count > 0 || skipped_count > 0 {
        "mail_projection_partial"
    } else {
        "mail_projection_available"
    };
    let summaries = artifacts
        .iter()
        .map(|artifact| artifact.summary.clone())
        .collect::<Vec<_>>();
    Some(MailboxRender {
        status: MailboxProfileStatus {
            profile: mode.name().to_string(),
            status: status.to_string(),
            output_root: mode.output_root().to_string(),
            message_count,
            emitted_message_count,
            artifact_count: artifacts.len(),
            skipped_count,
            unavailable_count,
            path_policy: "sanitized relative paths; folder collisions receive stable suffixes".to_string(),
            publication_policy: "archive entries are appended only after complete in-memory serialization; no partial direct files".to_string(),
            artifacts: summaries,
            decisions,
        },
        artifacts,
    })
}

fn ordered_mail_messages(messages: &[MessageRecord]) -> Vec<&MessageRecord> {
    let mut ordered = messages
        .iter()
        .filter(|message| message.item_type != "metadata_status")
        .collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        left.folder_path
            .cmp(&right.folder_path)
            .then(left.folder_key.cmp(&right.folder_key))
            .then(left.message_key.cmp(&right.message_key))
    });
    ordered
}

fn folder_path_map(
    folders: &[FolderRecord],
    messages: &[&MessageRecord],
) -> BTreeMap<(String, String), String> {
    let mut keys = BTreeSet::<(String, String)>::new();
    for folder in folders {
        keys.insert((folder.folder_key.clone(), folder.folder_path.clone()));
    }
    for message in messages {
        keys.insert((message.folder_key.clone(), message.folder_path.clone()));
    }
    let mut used = BTreeMap::<String, usize>::new();
    let mut result = BTreeMap::new();
    for (folder_key, folder_path) in keys {
        let base = safe_folder_path(&folder_path);
        let count = used.entry(base.clone()).or_default();
        *count += 1;
        let safe = if *count == 1 {
            base
        } else {
            format!("{base}_{:04}", *count)
        };
        result.insert((folder_key, folder_path), safe);
    }
    result
}

fn safe_folder_path(folder_path: &str) -> String {
    let segments = folder_path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(sanitize_segment)
        .collect::<Vec<_>>();
    if segments.is_empty() {
        "root".to_string()
    } else {
        segments.join("/")
    }
}

fn artifact(
    path: String,
    message_key: Option<String>,
    folder_path: String,
    output_kind: &str,
    bytes: Vec<u8>,
    status: &str,
) -> MailboxArtifact {
    MailboxArtifact {
        summary: MailboxArtifactSummary {
            path,
            message_key,
            folder_path,
            output_kind: output_kind.to_string(),
            sha256: sha256_hex(&bytes),
            size_bytes: bytes.len() as u64,
            status: status.to_string(),
        },
        bytes,
    }
}

fn serialize_message_eml(
    message: &MessageRecord,
    headers: &[HeaderProjectionRecord],
    recipients: &[RecipientRecord],
    bodies: &[BodyRecord],
    body_payloads: &[BodyPayload],
    attachments: &[AttachmentRecord],
    attachment_payloads: &[AttachmentPayload],
) -> Result<Vec<u8>, RenderError> {
    let payloads = body_payloads
        .iter()
        .filter(|payload| {
            payload.record.message_key == message.message_key
                && bodies
                    .iter()
                    .any(|body| body.body_key == payload.record.body_key)
        })
        .collect::<Vec<_>>();
    let text = payloads
        .iter()
        .find(|payload| payload.record.body_type == "text");
    let html = payloads
        .iter()
        .find(|payload| payload.record.body_type == "html");
    let report = payloads
        .iter()
        .find(|payload| payload.record.body_type == "report");
    let rtf = payloads
        .iter()
        .find(|payload| payload.record.body_type == "rtf");
    if text.is_none() && html.is_none() && report.is_none() && rtf.is_none() {
        return Err(RenderError {
            status: "unavailable_body_payload",
        });
    }

    let mut output = Vec::new();
    let header_record = headers
        .iter()
        .find(|header| header.message_key == message.message_key);
    let mut present = BTreeSet::new();
    if let Some(header_record) = header_record.filter(|header| header.authoritative) {
        if let Some(normalized) = header_record.normalized_headers.as_deref() {
            for line in filtered_header_lines(normalized) {
                if let Some(name) = line
                    .split_once(':')
                    .map(|(name, _)| name.to_ascii_lowercase())
                {
                    present.insert(name);
                }
                output.extend_from_slice(line.as_bytes());
                output.extend_from_slice(b"\r\n");
            }
        }
    }

    let sender = message
        .sender_email
        .as_deref()
        .or(message.sender_raw_address.as_deref())
        .and_then(clean_header)
        .unwrap_or_else(|| "MAILER-DAEMON".to_string());
    if !present.contains("from") {
        push_header(
            &mut output,
            "From",
            &format_address(message.sender_name.as_deref(), &sender),
        );
    }
    if !present.contains("to") {
        if let Some(value) = recipient_header(recipients, &message.message_key, "to") {
            push_header(&mut output, "To", &value);
        }
    }
    if !present.contains("cc") {
        if let Some(value) = recipient_header(recipients, &message.message_key, "cc") {
            push_header(&mut output, "Cc", &value);
        }
    }
    if !present.contains("subject") {
        push_header(
            &mut output,
            "Subject",
            &message
                .subject
                .as_deref()
                .and_then(clean_header)
                .unwrap_or_default(),
        );
    }
    if !present.contains("date") {
        if let Some(date) = message_date(message, header_record) {
            push_header(&mut output, "Date", &date);
        }
    }
    if !present.contains("message-id") {
        if let Some(message_id) = message
            .internet_message_id
            .as_deref()
            .and_then(clean_header)
        {
            push_header(&mut output, "Message-ID", &message_id);
        }
    }

    let boundary = ids::stable_id("boundary", &[&message.message_key, "mixed"]);
    let report_type = if report.is_some() {
        "; report-type=delivery-status"
    } else {
        ""
    };
    push_header(&mut output, "MIME-Version", "1.0");
    push_header(
        &mut output,
        "Content-Type",
        &format!(
            "multipart/{}{}; boundary=\"{}\"",
            if report.is_some() { "report" } else { "mixed" },
            report_type,
            boundary
        ),
    );
    output.extend_from_slice(b"\r\n");

    if let (Some(text), Some(html)) = (text, html) {
        let alternative = ids::stable_id("boundary", &[&message.message_key, "alternative"]);
        append_boundary(&mut output, &boundary, false);
        push_header(
            &mut output,
            "Content-Type",
            &format!("multipart/alternative; boundary=\"{}\"", alternative),
        );
        output.extend_from_slice(b"\r\n");
        append_text_part(&mut output, &alternative, "text/plain", &text.bytes);
        append_text_part(&mut output, &alternative, "text/html", &html.bytes);
        append_boundary(&mut output, &alternative, true);
    } else if let Some(body) = report.or(text).or(html) {
        let media_type = if report.is_some() || text.is_some() {
            "text/plain"
        } else {
            "text/html"
        };
        append_text_part(&mut output, &boundary, media_type, &body.bytes);
    }

    if let Some(rtf) = rtf {
        append_binary_part(
            &mut output,
            &boundary,
            "application/rtf",
            "rtf-body.rtf",
            &rtf.bytes,
            false,
            None,
        );
    }

    for attachment in attachments
        .iter()
        .filter(|attachment| attachment.message_key == message.message_key)
        .collect::<Vec<_>>()
    {
        let Some(payload) = attachment_payloads
            .iter()
            .find(|payload| payload.record.attachment_key == attachment.attachment_key)
        else {
            continue;
        };
        append_binary_part(
            &mut output,
            &boundary,
            attachment
                .content_type
                .as_deref()
                .unwrap_or("application/octet-stream"),
            &attachment.filename_safe,
            &payload.bytes,
            attachment.is_inline,
            attachment.content_id.as_deref(),
        );
    }
    append_boundary(&mut output, &boundary, true);
    Ok(output)
}

fn filtered_header_lines(normalized: &str) -> Vec<String> {
    let blocked = [
        "mime-version",
        "content-type",
        "content-transfer-encoding",
        "content-class",
        "x-mimeole",
        "x-from_",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let mut output = Vec::new();
    let mut keep = false;
    for line in normalized.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            if keep {
                output.push(line.to_string());
            }
            continue;
        }
        let Some((name, _)) = line.split_once(':') else {
            keep = false;
            continue;
        };
        keep = !blocked.contains(name.to_ascii_lowercase().as_str());
        if keep {
            output.push(line.to_string());
        }
    }
    output
}

fn append_text_part(output: &mut Vec<u8>, boundary: &str, media_type: &str, bytes: &[u8]) {
    append_boundary(output, boundary, false);
    push_header(
        output,
        "Content-Type",
        &format!("{}; charset=\"utf-8\"", media_type),
    );
    let transfer = text_transfer_encoding(bytes);
    push_header(output, "Content-Transfer-Encoding", transfer);
    output.extend_from_slice(b"\r\n");
    if transfer == "base64" {
        output.extend_from_slice(base64_encode(bytes).as_bytes());
        output.extend_from_slice(b"\r\n");
    } else {
        output.extend_from_slice(normalize_crlf_bytes(bytes).as_bytes());
        if !output.ends_with(b"\r\n") {
            output.extend_from_slice(b"\r\n");
        }
    }
}

fn append_binary_part(
    output: &mut Vec<u8>,
    boundary: &str,
    media_type: &str,
    filename: &str,
    bytes: &[u8],
    inline: bool,
    content_id: Option<&str>,
) {
    append_boundary(output, boundary, false);
    let safe_media_type = safe_media_type(media_type);
    push_header(
        output,
        "Content-Type",
        &format!("{}; name=\"{}\"", safe_media_type, quote_token(filename)),
    );
    push_header(output, "Content-Transfer-Encoding", "base64");
    push_header(
        output,
        "Content-Disposition",
        &format!(
            "{}; filename=\"{}\"",
            if inline { "inline" } else { "attachment" },
            quote_token(filename)
        ),
    );
    if let Some(content_id) = content_id.and_then(clean_header) {
        push_header(output, "Content-ID", &format!("<{}>", content_id));
    }
    output.extend_from_slice(b"\r\n");
    output.extend_from_slice(base64_encode(bytes).as_bytes());
    output.extend_from_slice(b"\r\n");
}

fn append_boundary(output: &mut Vec<u8>, boundary: &str, closing: bool) {
    output.extend_from_slice(b"--");
    output.extend_from_slice(boundary.as_bytes());
    if closing {
        output.extend_from_slice(b"--");
    }
    output.extend_from_slice(b"\r\n");
}

fn recipient_header(
    recipients: &[RecipientRecord],
    message_key: &str,
    role: &str,
) -> Option<String> {
    let values = recipients
        .iter()
        .filter(|recipient| {
            recipient.message_key == message_key && recipient.recipient_type == role
        })
        .filter_map(|recipient| {
            let address = recipient
                .smtp_address
                .as_deref()
                .or(recipient.raw_address.as_deref())
                .and_then(clean_header)?;
            Some(format_address(recipient.display_name.as_deref(), &address))
        })
        .collect::<Vec<_>>();
    (!values.is_empty()).then(|| values.join(", "))
}

fn message_date(
    message: &MessageRecord,
    header: Option<&HeaderProjectionRecord>,
) -> Option<String> {
    if let Some(normalized) = header.and_then(|record| record.normalized_headers.as_deref()) {
        if let Some(value) = header_value(normalized, "date") {
            return clean_header(&value);
        }
    }
    message
        .sent_at
        .as_deref()
        .and_then(filetime_date)
        .or_else(|| message.received_at.as_deref().and_then(filetime_date))
}

fn header_value(headers: &str, wanted: &str) -> Option<String> {
    headers.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case(wanted)
            .then(|| value.trim().to_string())
    })
}

fn filetime_date(value: &str) -> Option<String> {
    const FILETIME_UNIX_EPOCH_TICKS: u64 = 116_444_736_000_000_000;
    let ticks = value.strip_prefix("filetime:")?.parse::<u64>().ok()?;
    let unix_ticks = ticks.checked_sub(FILETIME_UNIX_EPOCH_TICKS)?;
    let seconds = i64::try_from(unix_ticks / 10_000_000).ok()?;
    let nanos = u32::try_from((unix_ticks % 10_000_000) * 100).ok()?;
    let date = chrono::DateTime::<chrono::Utc>::from_timestamp(seconds, nanos)?;
    Some(date.format("%a, %d %b %Y %H:%M:%S +0000").to_string())
}

fn mbox_separator(message: &MessageRecord) -> String {
    let sender = message
        .sender_email
        .as_deref()
        .or(message.sender_raw_address.as_deref())
        .and_then(clean_header)
        .unwrap_or_else(|| "MAILER-DAEMON".to_string());
    let date = message
        .sent_at
        .as_deref()
        .and_then(filetime_date)
        .unwrap_or_else(|| "Thu Jan 1 00:00:00 1970".to_string());
    format!("From {sender} {date}\n")
}

fn mboxrd_escape(bytes: &[u8]) -> Vec<u8> {
    let text = String::from_utf8_lossy(bytes).replace("\r\n", "\n");
    let mut output = String::new();
    for line in text.split_inclusive('\n') {
        let (content, newline) = line
            .strip_suffix('\n')
            .map_or((line, ""), |content| (content, "\n"));
        let mut prefix_count = 0usize;
        while content.as_bytes().get(prefix_count) == Some(&b'>') {
            prefix_count += 1;
        }
        if content[prefix_count..].starts_with("From ") {
            output.push('>');
        }
        output.push_str(content);
        output.push_str(newline);
    }
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output.into_bytes()
}

fn text_transfer_encoding(bytes: &[u8]) -> &'static str {
    if std::str::from_utf8(bytes).is_ok() && !bytes.contains(&0) {
        "8bit"
    } else {
        "base64"
    }
}

fn normalize_crlf_bytes(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\r\n")
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::new();
    let mut line_length = 0usize;
    for chunk in bytes.chunks(3) {
        let a = chunk[0];
        let b = *chunk.get(1).unwrap_or(&0);
        let c = *chunk.get(2).unwrap_or(&0);
        output.push(TABLE[(a >> 2) as usize] as char);
        output.push(TABLE[((a & 0x03) << 4 | b >> 4) as usize] as char);
        output.push(if chunk.len() > 1 {
            TABLE[((b & 0x0f) << 2 | c >> 6) as usize] as char
        } else {
            '='
        });
        output.push(if chunk.len() > 2 {
            TABLE[(c & 0x3f) as usize] as char
        } else {
            '='
        });
        line_length += 4;
        if line_length == 76 {
            output.push_str("\r\n");
            line_length = 0;
        }
    }
    output.trim_end_matches("\r\n").to_string()
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

fn format_address(name: Option<&str>, address: &str) -> String {
    match name.and_then(clean_header) {
        Some(name) => format!("{} <{}>", quote_display_name(&name), address),
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

fn quote_token(value: &str) -> String {
    sanitize_segment(value)
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
}

fn safe_media_type(value: &str) -> String {
    let value = value.trim();
    if value.contains(';') || value.contains('\r') || value.contains('\n') || !value.contains('/') {
        "application/octet-stream".to_string()
    } else {
        value.to_string()
    }
}

fn push_header(output: &mut Vec<u8>, name: &str, value: &str) {
    output.extend_from_slice(name.as_bytes());
    output.extend_from_slice(b": ");
    output.extend_from_slice(value.as_bytes());
    output.extend_from_slice(b"\r\n");
}

fn artifact_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn sha256_hex(bytes: &[u8]) -> String {
    artifact_sha256(bytes)
}

#[cfg(test)]
mod tests {
    use super::{mboxrd_escape, render_profile};
    use crate::config::OutputProfile;
    use crate::output::metadata::{BodyRecord, HeaderProjectionRecord, MessageRecord};
    use crate::pst::messages::text_body_payload;

    fn message(key: &str, folder: &str, class: Option<&str>) -> MessageRecord {
        MessageRecord {
            run_id: "run".to_string(),
            pst_id: "pst".to_string(),
            folder_key: format!("folder-{folder}"),
            message_key: key.to_string(),
            message_node_id: Some(key.to_string()),
            folder_path: folder.to_string(),
            item_type: "message_metadata".to_string(),
            message_class: class.map(ToString::to_string),
            subject: Some("subject".to_string()),
            sender_name: Some("Sender".to_string()),
            sender_email: Some("sender@example.test".to_string()),
            sender_raw_address: Some("sender@example.test".to_string()),
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
            transport_message_headers: None,
            internet_message_id: Some(format!("<{key}@example.test>")),
            in_reply_to_id: None,
            conversation_index: None,
            conversation_topic: None,
            normalized_subject: Some("subject".to_string()),
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

    fn header(message_key: &str) -> HeaderProjectionRecord {
        HeaderProjectionRecord {
            message_key: message_key.to_string(),
            header_key: "header".to_string(),
            source: "test".to_string(),
            charset_policy: "utf-8".to_string(),
            raw_evidence_key: None,
            raw_header_size_bytes: 0,
            raw_header_sha256: None,
            raw_header_bytes_hex: None,
            stored_headers: Some(
                "From: stored@example.test\r\nContent-Type: text/plain\r\nX-Test: yes\r\n"
                    .to_string(),
            ),
            normalized_headers: Some(
                "From: stored@example.test\nContent-Type: text/plain\nX-Test: yes".to_string(),
            ),
            validation_status: "stored_valid".to_string(),
            authoritative: true,
            status: "test".to_string(),
        }
    }

    #[test]
    fn emits_deterministic_mbox_and_message_file_profiles() {
        let msg = message("msg-1", "/Inbox", Some("IPM.Note"));
        let body = text_body_payload("msg-1", "From body\nsecond");
        let folders = Vec::new();
        let headers = vec![header("msg-1")];
        let mbox = render_profile(
            OutputProfile::Mbox,
            &folders,
            std::slice::from_ref(&msg),
            &headers,
            &[],
            std::slice::from_ref(&body.record),
            std::slice::from_ref(&body),
            &[],
            &[],
        )
        .expect("mbox profile");
        let repeat = render_profile(
            OutputProfile::Mbox,
            &folders,
            std::slice::from_ref(&msg),
            &headers,
            &[],
            std::slice::from_ref(&body.record),
            std::slice::from_ref(&body),
            &[],
            &[],
        )
        .expect("repeat profile");
        assert_eq!(mbox.status.status, "mail_projection_available");
        assert_eq!(mbox.artifacts[0].bytes, repeat.artifacts[0].bytes);
        assert!(String::from_utf8_lossy(&mbox.artifacts[0].bytes).contains(">From body"));

        let eml = render_profile(
            OutputProfile::Eml,
            &folders,
            std::slice::from_ref(&msg),
            &headers,
            &[],
            std::slice::from_ref(&body.record),
            std::slice::from_ref(&body),
            &[],
            &[],
        )
        .expect("eml profile");
        assert!(eml.artifacts[0].summary.path.ends_with("/1.eml"));
        assert!(!String::from_utf8_lossy(&eml.artifacts[0].bytes).starts_with("From sender"));
    }

    #[test]
    fn preserves_explicit_negative_statuses_without_empty_files() {
        let unsupported = message("msg-task", "/Inbox", Some("IPM.Task"));
        let missing = message("msg-missing", "/Inbox", None);
        let output = render_profile(
            OutputProfile::RecursiveMbox,
            &[],
            &[unsupported, missing],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
        )
        .expect("recursive profile");
        assert_eq!(output.artifacts.len(), 0);
        assert_eq!(output.status.skipped_count, 1);
        assert_eq!(output.status.unavailable_count, 1);
        assert_eq!(output.status.status, "mail_records_unavailable");
    }

    #[test]
    fn mboxrd_escapes_only_separator_like_lines() {
        assert_eq!(
            mboxrd_escape(b"From one\r\n>From two\r\nplain\r\n"),
            b">From one\n>>From two\nplain\n"
        );
    }

    #[test]
    fn header_with_mime_fields_is_normalized_without_duplicate_mime_headers() {
        let msg = message("msg-headers", "/Inbox", Some("IPM.Note"));
        let body = text_body_payload("msg-headers", "body");
        let output = render_profile(
            OutputProfile::Eml,
            &[],
            std::slice::from_ref(&msg),
            &[header("msg-headers")],
            &[],
            std::slice::from_ref(&body.record),
            std::slice::from_ref(&body),
            &[],
            &[],
        )
        .expect("eml profile");
        let eml = String::from_utf8(output.artifacts[0].bytes.clone()).expect("utf8 eml");
        assert_eq!(eml.matches("MIME-Version:").count(), 1);
        assert_eq!(eml.matches("Content-Type:").count(), 2);
        assert!(eml.contains("X-Test: yes\r\n"));
    }

    #[allow(dead_code)]
    fn _body_record(_: BodyRecord) {}
}
