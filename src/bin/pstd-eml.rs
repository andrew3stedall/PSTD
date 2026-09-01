use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};

use chrono::{DateTime, FixedOffset, Utc};
use pstd::engine::metadata::extract_metadata;
use pstd::output::headers::{
    encode_display_name, encode_mime_parameter, encode_unstructured_value, normalize_content_id,
};
use pstd::output::ids;
use pstd::output::metadata::{AttachmentRecord, MessageRecord, RecipientRecord};
use pstd::pst::attachments::AttachmentPayload;
use pstd::pst::messages::BodyPayload;
use sha2::{Digest, Sha256};

const LZFU_MAGIC: u32 = 0x7546_5a4c;
const MELA_MAGIC: u32 = 0x414c_454d;
const INITIAL_DICTIONARY: &[u8] = b"{\\rtf1\\ansi\\mac\\deff0\\deftab720{\\fonttbl;}{\\f0\\fnil \\froman \\fswiss \\fmodern \\fscript \\fdecor MS Sans SerifSymbolArialTimes New RomanCourier{\\colortbl\\red0\\green0\\blue0\r\n\\par \\pard\\plain\\f0\\fs20\\b\\i\\u\\tab\\tx";
const ALTERNATIVE_BOUNDARY: &str = "pstd-alternative-7f6a8d2b";
const MIXED_BOUNDARY: &str = "pstd-mixed-3e2b1a9c";
const FILETIME_UNIX_EPOCH_TICKS: u64 = 116_444_736_000_000_000;
const SKIP_DESTINATIONS: &[&str] = &[
    "fonttbl",
    "colortbl",
    "stylesheet",
    "info",
    "pict",
    "object",
    "generator",
];

fn main() {
    if let Err(error) = run() {
        eprintln!("pstd-eml: {error}");
        std::process::exit(1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AttachmentMode {
    Inline,
    External,
}

impl AttachmentMode {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "inline" => Ok(Self::Inline),
            "external" => Ok(Self::External),
            _ => Err(format!(
                "unsupported attachment mode {value:?}; expected inline or external"
            )),
        }
    }
}

fn run() -> Result<(), String> {
    let (input, output, attachment_mode) = parse_args()?;

    fs::create_dir_all(&output).map_err(|error| error.to_string())?;
    let input_display = input.display().to_string();
    let run_id = pstd::output::ids::run_id(&input_display);
    let pst_id = pstd::output::ids::pst_id(&input_display);
    let metadata = extract_metadata(&input_display, &run_id, &pst_id)
        .map_err(|error| format!("metadata extraction failed: {error}"))?;

    let recipients = recipients_by_message(&metadata.recipients);
    let bodies = bodies_by_message(&metadata.body_payloads);
    let attachments = attachments_by_message(&metadata.attachment_payloads);
    let synthetic_body_attachments = synthetic_body_attachments_by_message(&metadata.body_payloads);
    let attachment_records =
        attachment_records_by_message(&metadata.attachments, &metadata.attachment_payloads);
    let embedded_messages = embedded_message_keys(&metadata.attachments);
    let mut pending_external_attachments = Vec::new();
    let mut external_manifest = Vec::new();
    let mut external_paths = BTreeSet::new();
    external_paths.insert(PathBuf::from("attachments.jsonl"));
    let mut emitted = 0usize;
    for message in &metadata.messages {
        let message_recipients = recipients
            .get(&message.message_key)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let mut message_attachments = attachments
            .get(&message.message_key)
            .cloned()
            .unwrap_or_default();
        if let Some(synthetic) = synthetic_body_attachments.get(&message.message_key) {
            message_attachments.extend(synthetic.iter().cloned());
        }
        message_attachments.sort_by_key(|payload| attachment_order_key(&payload.record));
        let mut message_attachment_records = attachment_records
            .get(&message.message_key)
            .cloned()
            .unwrap_or_default();
        let existing_attachment_keys = message_attachment_records
            .iter()
            .map(|record| record.attachment_key.as_str())
            .collect::<BTreeSet<_>>();
        message_attachment_records.extend(
            message_attachments
                .iter()
                .filter(|payload| {
                    !existing_attachment_keys.contains(payload.record.attachment_key.as_str())
                })
                .map(|payload| payload.record.clone()),
        );
        message_attachment_records.sort_by_key(attachment_order_key);
        let unavailable_attachment_count =
            unavailable_attachment_count(&message_attachment_records, &message_attachments);
        let inline_attachments = if attachment_mode == AttachmentMode::Inline {
            message_attachments.as_slice()
        } else {
            &[]
        };
        let eml = bodies.get(&message.message_key).and_then(|body| {
            build_eml_with_attachment_mode(
                message,
                message_recipients,
                body,
                inline_attachments,
                attachment_mode,
                attachment_mode == AttachmentMode::External
                    || embedded_messages.contains(&message.message_key)
                    || unavailable_attachment_count > 0,
                unavailable_attachment_count,
            )
        });
        if attachment_mode == AttachmentMode::External {
            let eml_filename = format!("{}.eml", safe_filename(&message.message_key));
            if !external_paths.insert(PathBuf::from(eml_filename.as_str())) {
                return Err(format!("duplicate EML output path: {eml_filename}"));
            }
            let (pending, manifest) = plan_external_attachments(
                if eml.is_some() {
                    Some(eml_filename.as_str())
                } else {
                    None
                },
                message_attachment_records,
                message_attachments,
                &mut external_paths,
            )?;
            pending_external_attachments.extend(pending);
            external_manifest.extend(manifest);
        }
        let Some(eml) = eml else {
            continue;
        };
        let path = output.join(format!("{}.eml", safe_filename(&message.message_key)));
        fs::write(path, eml).map_err(|error| error.to_string())?;
        emitted += 1;
    }

    if attachment_mode == AttachmentMode::External {
        write_external_attachments(&output, &pending_external_attachments, &external_manifest)?;
    }

    println!("eml_files_emitted={emitted}");
    if emitted == 0 {
        return Err("no message had validated sender, recipients, subject, Date evidence, plain-text body, and a supported multipart body required for EML emission".to_string());
    }
    Ok(())
}

fn parse_args() -> Result<(PathBuf, PathBuf, AttachmentMode), String> {
    let mut positionals = Vec::new();
    let mut attachment_mode = AttachmentMode::Inline;
    let mut args = env::args_os().skip(1);

    while let Some(raw) = args.next() {
        let argument = raw.to_str().ok_or_else(usage)?;
        if argument == "--attachment-mode" {
            let Some(raw_value) = args.next() else {
                return Err(usage());
            };
            let value = raw_value.to_str().ok_or_else(usage)?;
            attachment_mode = AttachmentMode::parse(value)?;
        } else if let Some(value) = argument.strip_prefix("--attachment-mode=") {
            attachment_mode = AttachmentMode::parse(value)?;
        } else if argument.starts_with('-') {
            return Err(usage());
        } else {
            positionals.push(PathBuf::from(raw));
        }
    }

    if positionals.len() != 2 {
        return Err(usage());
    }
    Ok((
        positionals.remove(0),
        positionals.remove(0),
        attachment_mode,
    ))
}

fn usage() -> String {
    "usage: pstd-eml [--attachment-mode inline|external] <input.pst> <output-dir>".to_string()
}

#[derive(Default)]
struct MessageBodies {
    text: Option<Vec<u8>>,
    html: Option<String>,
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

fn attachments_by_message(
    payloads: &[AttachmentPayload],
) -> BTreeMap<String, Vec<AttachmentPayload>> {
    let mut grouped: BTreeMap<String, Vec<AttachmentPayload>> = BTreeMap::new();
    for payload in payloads {
        grouped
            .entry(payload.record.message_key.clone())
            .or_default()
            .push(payload.clone());
    }
    for payloads in grouped.values_mut() {
        payloads.sort_by_key(|payload| attachment_order_key(&payload.record));
    }
    grouped
}

fn synthetic_body_attachments_by_message(
    payloads: &[BodyPayload],
) -> BTreeMap<String, Vec<AttachmentPayload>> {
    let mut grouped = BTreeMap::<String, Vec<AttachmentPayload>>::new();
    let mut ordinals = BTreeMap::<String, u64>::new();
    for payload in payloads.iter().filter(|payload| {
        matches!(
            payload.record.body_type.as_str(),
            "rtf" | "encrypted" | "encrypted_html"
        )
    }) {
        let message_key = payload.record.message_key.clone();
        let ordinal = ordinals.entry(message_key.clone()).or_default();
        let synthetic = synthetic_body_attachment(payload, *ordinal);
        *ordinal = ordinal.saturating_add(1);
        grouped.entry(message_key).or_default().push(synthetic);
    }
    for payloads in grouped.values_mut() {
        payloads.sort_by_key(|payload| attachment_order_key(&payload.record));
    }
    grouped
}

fn synthetic_body_attachment(payload: &BodyPayload, ordinal: u64) -> AttachmentPayload {
    let (filename, content_type, status) = match payload.record.body_type.as_str() {
        "rtf" => (
            "rtf-body.rtf",
            "application/rtf",
            if payload.bytes.is_empty() {
                "synthetic_rtf_attachment_unavailable"
            } else if validated_rtf(&payload.bytes).is_some() {
                "synthetic_rtf_attachment_available"
            } else {
                "synthetic_rtf_attachment_invalid"
            },
        ),
        "encrypted_html" => (
            "encrypted-html-body.bin",
            "application/octet-stream",
            "encrypted_body_opaque",
        ),
        _ => (
            "encrypted-body.bin",
            "application/octet-stream",
            "encrypted_body_opaque",
        ),
    };
    let attachment_key = ids::stable_id(
        "att",
        &[
            &payload.record.message_key,
            "synthetic-body",
            &payload.record.body_key,
        ],
    );
    let archive_path = format!(
        "attachments/{}/{attachment_key}_{filename}",
        payload.record.message_key
    );
    let size_bytes = payload.bytes.len() as u64;
    AttachmentPayload {
        record: AttachmentRecord {
            message_key: payload.record.message_key.clone(),
            attachment_key,
            filename_original: Some(filename.to_string()),
            filename_safe: filename.to_string(),
            content_type: Some(content_type.to_string()),
            extension: Some(
                filename
                    .rsplit_once('.')
                    .map(|(_, extension)| extension.to_string())
                    .unwrap_or_default(),
            ),
            size_bytes,
            declared_size_bytes: Some(size_bytes),
            size_status: "size_matched".to_string(),
            sha256: sha256_hex(&payload.bytes),
            is_inline: false,
            is_hidden: false,
            content_id: None,
            attachment_method: None,
            source_ref: format!("body:{}", payload.record.body_type),
            rendering_position: Some(u64::MAX.saturating_sub(ordinal)),
            mime_sequence: None,
            embedded_message_key: None,
            ordinal: u64::MAX.saturating_sub(ordinal),
            archive_path,
            extraction_status: status.to_string(),
        },
        bytes: payload.bytes.clone(),
    }
}

fn attachment_records_by_message(
    records: &[AttachmentRecord],
    payloads: &[AttachmentPayload],
) -> BTreeMap<String, Vec<AttachmentRecord>> {
    let mut grouped: BTreeMap<String, Vec<AttachmentRecord>> = BTreeMap::new();
    for record in records {
        grouped
            .entry(record.message_key.clone())
            .or_default()
            .push(record.clone());
    }
    for payload in payloads {
        let records = grouped
            .entry(payload.record.message_key.clone())
            .or_default();
        if !records
            .iter()
            .any(|record| record.attachment_key == payload.record.attachment_key)
        {
            records.push(payload.record.clone());
        }
    }
    for records in grouped.values_mut() {
        records.sort_by_key(attachment_order_key);
    }
    grouped
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

fn embedded_message_keys(records: &[AttachmentRecord]) -> BTreeSet<String> {
    records
        .iter()
        .filter_map(|record| record.embedded_message_key.clone())
        .collect()
}

fn unavailable_attachment_count(
    records: &[AttachmentRecord],
    payloads: &[AttachmentPayload],
) -> usize {
    records
        .iter()
        .filter(|record| {
            match payloads
                .iter()
                .find(|payload| payload.record.attachment_key == record.attachment_key)
            {
                Some(payload) => !payload_matches_record(payload, record),
                None => true,
            }
        })
        .count()
}

fn payload_matches_record(payload: &AttachmentPayload, record: &AttachmentRecord) -> bool {
    payload.record.message_key == record.message_key
        && payload.record.attachment_key == record.attachment_key
        && payload.record.archive_path == record.archive_path
        && record.size_bytes == payload.bytes.len() as u64
        && record.sha256 == sha256_hex(&payload.bytes)
        && payload.record.size_bytes == payload.bytes.len() as u64
        && payload.record.sha256 == sha256_hex(&payload.bytes)
}

struct PendingExternalAttachment {
    relative_path: PathBuf,
    bytes: Vec<u8>,
}

fn plan_external_attachments(
    eml_path: Option<&str>,
    records: &[AttachmentRecord],
    payloads: &[AttachmentPayload],
    used_paths: &mut BTreeSet<PathBuf>,
) -> Result<(Vec<PendingExternalAttachment>, Vec<String>), String> {
    let mut pending = Vec::new();
    let mut manifest = Vec::new();
    for record in records {
        let relative_path = relative_attachment_path(record)?;
        if !used_paths.insert(relative_path.clone()) {
            return Err(format!(
                "duplicate external attachment path: {}",
                record.archive_path
            ));
        }

        let matching_payloads = payloads
            .iter()
            .filter(|payload| payload.record.attachment_key == record.attachment_key)
            .collect::<Vec<_>>();
        if matching_payloads.len() > 1 {
            return Err(format!(
                "duplicate attachment payload key: {}",
                record.attachment_key
            ));
        }
        let (materialization_status, materialized_path, payload_bytes) =
            match matching_payloads.first().copied() {
                Some(payload) if payload_matches_record(payload, record) => (
                    "attachment_file_emitted",
                    Some(record.archive_path.clone()),
                    Some(payload.bytes.clone()),
                ),
                Some(_) => ("attachment_payload_integrity_failed", None, None),
                None => ("attachment_payload_unavailable", None, None),
            };
        if let Some(bytes) = payload_bytes {
            pending.push(PendingExternalAttachment {
                relative_path,
                bytes,
            });
        }

        let mut value = serde_json::to_value(record)
            .map_err(|error| format!("failed to serialize attachment record: {error}"))?;
        let Some(object) = value.as_object_mut() else {
            return Err("attachment record did not serialize as a JSON object".to_string());
        };
        object.insert(
            "eml_path".to_string(),
            eml_path
                .map(ToString::to_string)
                .map(serde_json::Value::String)
                .unwrap_or(serde_json::Value::Null),
        );
        object.insert(
            "materialized_path".to_string(),
            materialized_path
                .map(serde_json::Value::String)
                .unwrap_or(serde_json::Value::Null),
        );
        object.insert(
            "materialization_status".to_string(),
            serde_json::Value::String(materialization_status.to_string()),
        );
        manifest.push(
            serde_json::to_string(&value)
                .map_err(|error| format!("failed to serialize attachment manifest: {error}"))?,
        );
    }
    Ok((pending, manifest))
}

fn relative_attachment_path(record: &AttachmentRecord) -> Result<PathBuf, String> {
    let path = Path::new(&record.archive_path);
    if path.as_os_str().is_empty()
        || record.archive_path.contains('\\')
        || record.archive_path.contains('\0')
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::CurDir
                    | Component::ParentDir
                    | Component::RootDir
                    | Component::Prefix(_)
            )
        })
    {
        return Err(format!(
            "attachment archive path is not confined to the output directory: {}",
            record.archive_path
        ));
    }
    Ok(path.to_path_buf())
}

fn write_external_attachments(
    output: &Path,
    pending: &[PendingExternalAttachment],
    manifest: &[String],
) -> Result<(), String> {
    for attachment in pending {
        let path = output.join(&attachment.relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create external attachment directory {}: {error}",
                    parent.display()
                )
            })?;
        }
        fs::write(&path, &attachment.bytes).map_err(|error| {
            format!(
                "failed to write external attachment {}: {error}",
                attachment.relative_path.display()
            )
        })?;
    }

    let mut manifest_bytes = String::new();
    for line in manifest {
        manifest_bytes.push_str(line);
        manifest_bytes.push('\n');
    }
    fs::write(output.join("attachments.jsonl"), manifest_bytes)
        .map_err(|error| format!("failed to write attachments.jsonl: {error}"))
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
            "html" if entry.html.is_none() => {
                entry.html = String::from_utf8(payload.bytes.clone())
                    .ok()
                    .filter(|value| !value.is_empty());
            }
            "rtf" if entry.html.is_none() => {
                entry.html = validated_rtf(&payload.bytes)
                    .and_then(|rtf| String::from_utf8(rtf).ok())
                    .and_then(|rtf| recover_html(&rtf));
            }
            _ => {}
        }
    }
    bodies.retain(|_, body| body.text.is_some());
    bodies
}

#[cfg(test)]
fn build_eml(
    message: &MessageRecord,
    recipients: &[RecipientRecord],
    bodies: &MessageBodies,
    attachments: &[AttachmentPayload],
) -> Option<Vec<u8>> {
    build_eml_with_attachment_mode(
        message,
        recipients,
        bodies,
        attachments,
        AttachmentMode::Inline,
        false,
        0,
    )
}

#[cfg(test)]
fn build_eml_with_plain_text_policy(
    message: &MessageRecord,
    recipients: &[RecipientRecord],
    bodies: &MessageBodies,
    attachments: &[AttachmentPayload],
    allow_plain_text_only: bool,
) -> Option<Vec<u8>> {
    build_eml_with_attachment_mode(
        message,
        recipients,
        bodies,
        attachments,
        AttachmentMode::Inline,
        allow_plain_text_only,
        0,
    )
}

fn build_eml_with_attachment_mode(
    message: &MessageRecord,
    recipients: &[RecipientRecord],
    bodies: &MessageBodies,
    attachments: &[AttachmentPayload],
    attachment_mode: AttachmentMode,
    allow_plain_text_only: bool,
    unavailable_attachment_count: usize,
) -> Option<Vec<u8>> {
    let subject = clean_header(message.subject.as_deref()?)?;
    let sender_address = message
        .sender_email
        .as_deref()
        .or(message.sender_raw_address.as_deref())
        .and_then(clean_header)?;
    let sender_name = message.sender_name.as_deref().and_then(clean_header);
    let from = format_address(sender_name.as_deref(), &sender_address);
    let to =
        recipient_header(recipients, "to").or_else(|| transport_recipient_header(message, "To"));
    let cc =
        recipient_header(recipients, "cc").or_else(|| transport_recipient_header(message, "Cc"));
    if to.is_none() && cc.is_none() {
        return None;
    }
    let date = validated_message_date(message)?;

    let text = std::str::from_utf8(bodies.text.as_deref()?).ok()?;
    let html = bodies.html.as_deref();
    if text.contains(ALTERNATIVE_BOUNDARY)
        || text.contains(MIXED_BOUNDARY)
        || html.is_some_and(|value| {
            value.contains(ALTERNATIVE_BOUNDARY) || value.contains(MIXED_BOUNDARY)
        })
    {
        return None;
    }
    if attachment_mode == AttachmentMode::Inline
        && !attachments.is_empty()
        && !attachments_are_valid(attachments)
    {
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
    if attachment_mode == AttachmentMode::External {
        push_header(&mut eml, "X-PSTD-Attachment-Mode", "external");
        push_header(&mut eml, "X-PSTD-Attachment-Manifest", "attachments.jsonl");
        push_header(&mut eml, "X-PSTD-Attachment-Root", ".");
    }
    if unavailable_attachment_count > 0 {
        push_header(
            &mut eml,
            "X-PSTD-Attachments-Unavailable",
            &unavailable_attachment_count.to_string(),
        );
    }
    push_header(&mut eml, "MIME-Version", "1.0");

    if attachments.is_empty() {
        if let Some(html) = html {
            push_header(
                &mut eml,
                "Content-Type",
                &format!("multipart/alternative; boundary=\"{ALTERNATIVE_BOUNDARY}\""),
            );
            eml.push_str("\r\n");
            push_alternative_body(&mut eml, text, html);
        } else {
            if !allow_plain_text_only {
                return None;
            }
            push_header(&mut eml, "Content-Type", "text/plain; charset=utf-8");
            push_header(&mut eml, "Content-Transfer-Encoding", "8bit");
            eml.push_str("\r\n");
            eml.push_str(&normalize_crlf(text));
            if !eml.ends_with("\r\n") {
                eml.push_str("\r\n");
            }
        }
    } else {
        push_header(
            &mut eml,
            "Content-Type",
            &format!("multipart/mixed; boundary=\"{MIXED_BOUNDARY}\""),
        );
        eml.push_str("\r\n");
        eml.push_str("--");
        eml.push_str(MIXED_BOUNDARY);
        eml.push_str("\r\n");
        if let Some(html) = html {
            push_header(
                &mut eml,
                "Content-Type",
                &format!("multipart/alternative; boundary=\"{ALTERNATIVE_BOUNDARY}\""),
            );
            eml.push_str("\r\n");
            push_alternative_body(&mut eml, text, html);
        } else {
            push_header(&mut eml, "Content-Type", "text/plain; charset=utf-8");
            push_header(&mut eml, "Content-Transfer-Encoding", "8bit");
            eml.push_str("\r\n");
            eml.push_str(&normalize_crlf(text));
            if !eml.ends_with("\r\n") {
                eml.push_str("\r\n");
            }
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

fn push_alternative_body(output: &mut String, text: &str, html: &str) {
    push_part(output, "text/plain; charset=utf-8", &normalize_crlf(text));
    push_part(output, "text/html; charset=utf-8", &normalize_crlf(html));
    output.push_str("--");
    output.push_str(ALTERNATIVE_BOUNDARY);
    output.push_str("--\r\n");
}

fn attachments_are_valid(attachments: &[AttachmentPayload]) -> bool {
    let mut attachment_keys = BTreeSet::new();
    let mut ordinals = BTreeSet::new();
    attachments.iter().all(|attachment| {
        payload_is_valid(attachment)
            && attachment_keys.insert(attachment.record.attachment_key.clone())
            && ordinals.insert(attachment.record.ordinal)
    })
}

fn payload_is_valid(attachment: &AttachmentPayload) -> bool {
    attachment.record.size_bytes == attachment.bytes.len() as u64
        && attachment.record.sha256 == sha256_hex(&attachment.bytes)
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

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
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

#[derive(Clone, Copy)]
struct HtmlState {
    skip: bool,
    htmltag: bool,
    ignorable: bool,
}

fn recover_html(rtf: &str) -> Option<String> {
    if !rtf.starts_with("{\\rtf") || !rtf.contains("\\fromhtml1") {
        return None;
    }
    let bytes = rtf.as_bytes();
    let mut output = String::new();
    let mut stack = vec![HtmlState {
        skip: false,
        htmltag: false,
        ignorable: false,
    }];
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'{' => {
                let mut state = *stack.last()?;
                state.ignorable = false;
                stack.push(state);
                index += 1;
            }
            b'}' => {
                if stack.len() == 1 {
                    return None;
                }
                stack.pop();
                index += 1;
            }
            b'\\' => {
                let (word, number, next) = read_control(bytes, index)?;
                index = next;
                let state = stack.last_mut()?;
                match word.as_str() {
                    "*" => state.ignorable = true,
                    "htmltag" => {
                        state.htmltag = true;
                        state.skip = false;
                    }
                    "htmlrtf" => state.skip = true,
                    "par" | "line" if !state.skip => output.push('\n'),
                    "tab" if !state.skip => output.push('\t'),
                    "hex" if !state.skip => output.push(cp1252_char(number? as u8)?),
                    "{" | "}" | "\\" if !state.skip => output.push_str(&word),
                    destination if SKIP_DESTINATIONS.contains(&destination) => state.skip = true,
                    _ if state.ignorable && !state.htmltag => state.skip = true,
                    _ => {}
                }
            }
            byte => {
                if !stack.last()?.skip && byte != b'\r' && byte != b'\n' {
                    output.push(byte as char);
                }
                index += 1;
            }
        }
    }
    if stack.len() != 1 {
        return None;
    }
    let html = output.trim().to_string();
    if html.is_empty()
        || !html.contains('<')
        || !html.contains('>')
        || html.contains("{\\rtf")
        || html.contains("\\htmltag")
        || html.contains('\\')
    {
        return None;
    }
    Some(html)
}

fn read_control(input: &[u8], start: usize) -> Option<(String, Option<i32>, usize)> {
    let mut index = start.checked_add(1)?;
    let first = *input.get(index)?;
    if matches!(first, b'{' | b'}' | b'\\') {
        return Some(((first as char).to_string(), None, index + 1));
    }
    if first == b'\'' {
        let hex = std::str::from_utf8(input.get(index + 1..index + 3)?).ok()?;
        let value = u8::from_str_radix(hex, 16).ok()?;
        return Some(("hex".to_string(), Some(i32::from(value)), index + 3));
    }
    if !first.is_ascii_alphabetic() {
        return Some(((first as char).to_string(), None, index + 1));
    }
    let word_start = index;
    while input.get(index).is_some_and(u8::is_ascii_alphabetic) {
        index += 1;
    }
    let word = std::str::from_utf8(input.get(word_start..index)?)
        .ok()?
        .to_string();
    let mut sign = 1i32;
    if input.get(index) == Some(&b'-') {
        sign = -1;
        index += 1;
    }
    let number_start = index;
    while input.get(index).is_some_and(u8::is_ascii_digit) {
        index += 1;
    }
    let number = if index > number_start {
        Some(
            std::str::from_utf8(input.get(number_start..index)?)
                .ok()?
                .parse::<i32>()
                .ok()?
                * sign,
        )
    } else {
        None
    };
    if input.get(index) == Some(&b' ') {
        index += 1;
    }
    Some((word, number, index))
}

fn cp1252_char(value: u8) -> Option<char> {
    match value {
        0x00..=0x7f | 0xa0..=0xff => char::from_u32(u32::from(value)),
        0x80 => Some('€'),
        0x82 => Some('‚'),
        0x83 => Some('ƒ'),
        0x84 => Some('„'),
        0x85 => Some('…'),
        0x86 => Some('†'),
        0x87 => Some('‡'),
        0x88 => Some('ˆ'),
        0x89 => Some('‰'),
        0x8a => Some('Š'),
        0x8b => Some('‹'),
        0x8c => Some('Œ'),
        0x8e => Some('Ž'),
        0x91 => Some('‘'),
        0x92 => Some('’'),
        0x93 => Some('“'),
        0x94 => Some('”'),
        0x95 => Some('•'),
        0x96 => Some('–'),
        0x97 => Some('—'),
        0x98 => Some('˜'),
        0x99 => Some('™'),
        0x9a => Some('š'),
        0x9b => Some('›'),
        0x9c => Some('œ'),
        0x9e => Some('ž'),
        0x9f => Some('Ÿ'),
        _ => None,
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

fn transport_recipient_header(message: &MessageRecord, target: &str) -> Option<String> {
    let headers = message.transport_message_headers.as_deref()?;
    let mut values = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_value = String::new();

    for line in headers.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            if current_name.is_some() {
                current_value.push(' ');
                current_value.push_str(line.trim());
            }
            continue;
        }

        if let Some(name) = current_name.take() {
            if name.eq_ignore_ascii_case(target) {
                values.push(current_value.trim().to_string());
            }
        }
        let (name, value) = line.split_once(':')?;
        current_name = Some(name.to_string());
        current_value = value.trim().to_string();
    }

    if let Some(name) = current_name {
        if name.eq_ignore_ascii_case(target) {
            values.push(current_value.trim().to_string());
        }
    }

    (values.len() == 1)
        .then(|| clean_header(&values[0]))
        .flatten()
}

fn format_address(name: Option<&str>, address: &str) -> String {
    match name.filter(|name| !name.is_empty()) {
        Some(name) => format!("{} <{}>", encode_display_name(name), address),
        None => address.to_string(),
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
    use pstd::pst::attachments::{
        attachment_payload, unavailable_attachment_record, AttachmentMetadata,
    };
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
            message_class: None,
            subject: Some("Fixture subject".to_string()),
            sender_name: Some("Fixture Sender".to_string()),
            sender_email: Some("sender@example.com".to_string()),
            sender_raw_address: None,
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
    fn emits_multipart_plain_and_html_message() {
        let bodies = MessageBodies {
            text: Some(b"Hello\nworld".to_vec()),
            html: Some("<b>Rich body</b>".to_string()),
        };
        let eml = build_eml(
            &message(),
            &[recipient(0, "to"), recipient(1, "cc")],
            &bodies,
            &[],
        )
        .unwrap();
        let eml = String::from_utf8(eml).unwrap();
        assert!(eml.contains(
            "Content-Type: multipart/alternative; boundary=\"pstd-alternative-7f6a8d2b\"\r\n"
        ));
        assert!(eml.contains("Content-Type: text/plain; charset=utf-8\r\n"));
        assert!(eml.contains("Content-Type: text/html; charset=utf-8\r\n"));
        assert!(!eml.contains("Content-Type: text/rtf"));
        assert!(eml.contains("Hello\r\nworld"));
        assert!(eml.contains("<b>Rich body</b>"));
        assert!(eml.ends_with("--pstd-alternative-7f6a8d2b--\r\n"));
    }

    #[test]
    fn recovers_html_from_validated_fromhtml_rtf() {
        let rtf = "{\\rtf1\\ansi\\fromhtml1{\\*\\htmltag <b>}Bold{\\*\\htmltag </b>}}";
        assert_eq!(recover_html(rtf).as_deref(), Some("<b>Bold</b>"));
    }

    #[test]
    fn rejects_non_html_and_malformed_rtf() {
        assert!(recover_html("{\\rtf1 plain}").is_none());
        assert!(recover_html("{\\rtf1\\fromhtml1{\\*\\htmltag <b>}Bold").is_none());
        assert!(recover_html("{\\rtf1\\fromhtml1 no markup}").is_none());
    }

    #[test]
    fn groups_messages_with_plain_text_and_optional_recoverable_html() {
        let payloads = vec![
            body_payload("message", "text", b"plain".to_vec(), None),
            body_payload(
                "message",
                "rtf",
                b"{\\rtf1\\fromhtml1{\\*\\htmltag <b>}rich{\\*\\htmltag </b>}}".to_vec(),
                None,
            ),
            body_payload("other", "text", b"plain".to_vec(), None),
        ];
        let grouped = bodies_by_message(&payloads);
        assert!(grouped.contains_key("message"));
        assert!(grouped.get("message").unwrap().html.is_some());
        assert!(grouped.contains_key("other"));
        assert!(grouped.get("other").unwrap().html.is_none());
    }

    #[test]
    fn accepts_extracted_html_body_payload() {
        let payloads = vec![
            body_payload("message", "html", b"<p>direct HTML</p>".to_vec(), None),
            body_payload("message", "text", b"plain text".to_vec(), None),
        ];

        let grouped = bodies_by_message(&payloads);

        assert_eq!(
            grouped.get("message").and_then(|body| body.html.as_deref()),
            Some("<p>direct HTML</p>")
        );
    }

    #[test]
    fn emits_single_part_plain_text_without_html_or_attachments() {
        let bodies = MessageBodies {
            text: Some(b"plain\nbody".to_vec()),
            html: None,
        };
        assert!(build_eml(&message(), &[recipient(0, "to")], &bodies, &[]).is_none());
        let eml =
            build_eml_with_plain_text_policy(&message(), &[recipient(0, "to")], &bodies, &[], true)
                .unwrap();
        let eml = String::from_utf8(eml).unwrap();

        assert!(eml.contains("Content-Type: text/plain; charset=utf-8\r\n"));
        assert!(eml.contains("Content-Transfer-Encoding: 8bit\r\n"));
        assert!(eml.contains("\r\n\r\nplain\r\nbody\r\n"));
        assert!(!eml.contains("multipart/alternative"));
        assert!(!eml.contains(ALTERNATIVE_BOUNDARY));
        assert!(!eml.contains("Content-Type: text/html"));
    }

    #[test]
    fn fails_closed_for_boundary_collision() {
        let recipients = vec![recipient(0, "to")];
        let collision = MessageBodies {
            text: Some(ALTERNATIVE_BOUNDARY.as_bytes().to_vec()),
            html: Some("<b>rich</b>".to_string()),
        };
        assert!(build_eml(&message(), &recipients, &collision, &[]).is_none());
    }

    #[test]
    fn includes_recovered_embedded_message_payloads_in_mime_assembly() {
        let by_value = attachment(0, b"docx");
        let mut embedded = attachment(1, b"child eml");
        embedded.record.attachment_method = Some(5);
        embedded.record.content_type = Some("message/rfc822".to_string());

        let grouped = attachments_by_message(&[by_value, embedded]);
        let payloads = grouped.get("message").unwrap();
        assert_eq!(payloads.len(), 2);
        assert_eq!(payloads[0].record.attachment_method, Some(1));
        assert_eq!(payloads[1].record.attachment_method, Some(5));
    }

    fn attachment(ordinal: usize, bytes: &[u8]) -> AttachmentPayload {
        attachment_payload(
            "message",
            ordinal,
            AttachmentMetadata {
                filename_original: Some("attachment.docx".to_string()),
                content_type: None,
                is_inline: false,
                attachment_method: Some(1),
                declared_size_bytes: Some(bytes.len() as u64),
                ..AttachmentMetadata::default()
            },
            bytes.to_vec(),
        )
    }

    #[test]
    fn emits_multipart_mixed_with_plain_body_and_attachment() {
        let mut message = message();
        message.transport_message_headers = None;
        message.received_at = Some("filetime:132509026800000000".to_string());
        let bodies = MessageBodies {
            text: Some("Forwarding mail…\r\n\r\n".as_bytes().to_vec()),
            html: None,
        };
        let attachments = vec![attachment(0, b"Hello attachment")];
        let eml = build_eml(&message, &[recipient(0, "to")], &bodies, &attachments).unwrap();
        let eml = String::from_utf8(eml).unwrap();

        assert!(eml.contains("Date: Thu, 26 Nov 2020 22:18:00 +0000\r\n"));
        assert!(eml.contains("Content-Type: multipart/mixed; boundary=\"pstd-mixed-3e2b1a9c\"\r\n"));
        assert!(eml.contains("Content-Type: text/plain; charset=utf-8\r\n"));
        assert!(eml.contains(
            "Content-Type: application/vnd.openxmlformats-officedocument.wordprocessingml.document; name=\"attachment.docx\"\r\n"
        ));
        assert!(eml.contains("Content-Disposition: attachment; filename=\"attachment.docx\"\r\n"));
        assert!(eml.contains("SGVsbG8gYXR0YWNobWVudA==\r\n"));
        assert!(eml.ends_with("--pstd-mixed-3e2b1a9c--\r\n"));
    }

    #[test]
    fn emits_inline_content_id_and_preserves_html_reference() {
        let mut message = message();
        message.transport_message_headers = None;
        message.received_at = Some("filetime:132509026800000000".to_string());
        let bodies = MessageBodies {
            text: Some(b"plain body".to_vec()),
            html: Some("<p><img src=\"cid:image-1@example.com\"></p>".to_string()),
        };
        let mut inline = attachment(0, b"image bytes");
        inline.record.filename_safe = "image.png".to_string();
        inline.record.extension = Some("png".to_string());
        inline.record.content_type = Some("image/png".to_string());
        inline.record.is_inline = true;
        inline.record.content_id = Some("image-1@example.com".to_string());

        let eml = build_eml(&message, &[recipient(0, "to")], &bodies, &[inline]).unwrap();
        let eml = String::from_utf8(eml).unwrap();
        assert!(eml.contains("cid:image-1@example.com"));
        assert!(eml.contains("Content-Disposition: inline; filename=\"image.png\"\r\n"));
        assert!(eml.contains("Content-ID: <image-1@example.com>\r\n"));
    }

    #[test]
    fn falls_back_to_one_validated_transport_recipient_header() {
        let mut message = message();
        message.transport_message_headers =
            Some("Date: 19 Aug 2015 11:07:26 +0000\r\nTo: raw@example.com\r\n".to_string());
        let bodies = MessageBodies {
            text: Some(b"plain body".to_vec()),
            html: Some("<p>rich body</p>".to_string()),
        };

        let eml = build_eml(&message, &[], &bodies, &[]).unwrap();
        let eml = String::from_utf8(eml).unwrap();
        assert!(eml.contains("To: raw@example.com\r\n"));
    }

    #[test]
    fn rejects_mixed_eml_without_date_or_valid_payload() {
        let mut message = message();
        message.transport_message_headers = None;
        message.received_at = None;
        let bodies = MessageBodies {
            text: Some(b"plain".to_vec()),
            html: None,
        };
        let recipient = recipient(0, "to");
        let attachment = attachment(0, b"bytes");
        assert!(build_eml(
            &message,
            std::slice::from_ref(&recipient),
            &bodies,
            std::slice::from_ref(&attachment),
        )
        .is_none());

        message.received_at = Some("filetime:132509026800000000".to_string());
        let mut invalid = attachment;
        invalid.record.sha256 = "not-the-payload-hash".to_string();
        assert!(build_eml(&message, &[recipient], &bodies, &[invalid]).is_none());
    }

    #[test]
    fn emits_recovered_embedded_message_as_rfc822_attachment() {
        let mut message = message();
        message.transport_message_headers = None;
        message.received_at = Some("filetime:132509026800000000".to_string());
        let bodies = MessageBodies {
            text: Some(b"plain".to_vec()),
            html: None,
        };
        let mut embedded = attachment(0, b"child eml");
        embedded.record.attachment_method = Some(5);
        embedded.record.content_type = None;
        embedded.record.filename_safe = "child.eml".to_string();
        embedded.record.extension = Some("eml".to_string());
        let eml = build_eml(&message, &[recipient(0, "to")], &bodies, &[embedded]).unwrap();
        let eml = String::from_utf8(eml).unwrap();
        assert!(eml.contains("Content-Type: message/rfc822; name=\"child.eml\"\r\n"));
        assert!(eml.contains("Y2hpbGQgZW1s\r\n"));
    }

    #[test]
    fn retains_zero_length_attachment_as_an_empty_mime_part() {
        let mut message = message();
        message.transport_message_headers = None;
        message.received_at = Some("filetime:132509026800000000".to_string());
        let bodies = MessageBodies {
            text: Some(b"plain".to_vec()),
            html: None,
        };
        let empty = attachment(0, &[]);
        let eml = build_eml(&message, &[recipient(0, "to")], &bodies, &[empty]).unwrap();
        let eml = String::from_utf8(eml).unwrap();
        assert!(eml.contains("Content-Disposition: attachment; filename=\"attachment.docx\"\r\n"));
        assert!(eml.contains(
            "Content-Disposition: attachment; filename=\"attachment.docx\"\r\n\r\n--pstd-mixed-3e2b1a9c--"
        ));
    }

    #[test]
    fn materializes_synthetic_rtf_and_encrypted_body_payloads() {
        let payloads = vec![
            body_payload("message", "rtf", b"{\\rtf1\\ansi body}".to_vec(), None),
            body_payload("message", "encrypted_html", Vec::new(), None),
        ];
        let grouped = synthetic_body_attachments_by_message(&payloads);
        let attachments = grouped.get("message").unwrap();

        assert_eq!(attachments.len(), 2);
        assert_eq!(attachments[0].record.filename_safe, "rtf-body.rtf");
        assert_eq!(
            attachments[0].record.extraction_status,
            "synthetic_rtf_attachment_available"
        );
        assert_eq!(attachments[0].bytes, b"{\\rtf1\\ansi body}");
        assert_eq!(attachments[1].record.filename_safe, "encrypted-html-body.bin");
        assert_eq!(attachments[1].record.extraction_status, "encrypted_body_opaque");
        assert!(attachments[1].bytes.is_empty());
        assert_ne!(
            attachments[0].record.attachment_key,
            attachments[1].record.attachment_key
        );
        assert!(attachments.iter().all(payload_is_valid));
    }

    #[test]
    fn emits_synthetic_body_payloads_as_inline_base64_attachments() {
        let bodies = MessageBodies {
            text: Some(b"plain body".to_vec()),
            html: None,
        };
        let payloads = vec![
            body_payload("message", "rtf", b"{\\rtf1\\ansi body}".to_vec(), None),
            body_payload("message", "encrypted", b"opaque bytes".to_vec(), None),
        ];
        let grouped = synthetic_body_attachments_by_message(&payloads);
        let attachments = grouped.get("message").unwrap();
        let eml = build_eml(
            &message(),
            &[recipient(0, "to")],
            &bodies,
            attachments,
        )
        .unwrap();
        let eml = String::from_utf8(eml).unwrap();

        assert!(eml.contains(
            "Content-Type: application/rtf; name=\"rtf-body.rtf\"\r\n"
        ));
        assert!(eml.contains("Content-Disposition: attachment; filename=\"rtf-body.rtf\"\r\n"));
        assert!(eml.contains("e1x0ZjFcYW5zaSBib2R5\r\n"));
        assert!(eml.contains(
            "Content-Type: application/octet-stream; name=\"encrypted-body.bin\"\r\n"
        ));
        assert!(eml.contains("b3BhcXVlIGJ5dGVz\r\n"));
    }

    #[test]
    fn groups_attachment_payloads_by_message_and_ordinal() {
        let mut second_message = attachment(0, b"other");
        second_message.record.message_key = "other".to_string();
        let grouped = attachments_by_message(&[
            attachment(2, b"second"),
            second_message,
            attachment(1, b"first"),
        ]);
        let message = grouped.get("message").unwrap();
        assert_eq!(message.len(), 2);
        assert_eq!(message[0].record.ordinal, 1);
        assert_eq!(message[1].record.ordinal, 2);
        assert_eq!(grouped.get("other").unwrap().len(), 1);
    }

    #[test]
    fn orders_attachment_payloads_by_mime_sequence_before_ordinal() {
        let mut first = attachment(0, b"first");
        first.record.mime_sequence = Some(2);
        let mut second = attachment(1, b"second");
        second.record.mime_sequence = Some(1);
        let grouped = attachments_by_message(&[first, second]);
        let payloads = grouped.get("message").unwrap();
        assert_eq!(payloads[0].record.ordinal, 1);
        assert_eq!(payloads[1].record.ordinal, 0);
        assert!(attachments_are_valid(payloads));
    }

    #[test]
    fn parses_attachment_modes() {
        assert_eq!(AttachmentMode::parse("inline"), Ok(AttachmentMode::Inline));
        assert_eq!(
            AttachmentMode::parse("external"),
            Ok(AttachmentMode::External)
        );
        assert!(AttachmentMode::parse("sidecar").is_err());
    }

    #[test]
    fn external_mode_emits_manifest_headers_without_inline_base64() {
        let bodies = MessageBodies {
            text: Some(b"plain body".to_vec()),
            html: None,
        };
        let eml = build_eml_with_attachment_mode(
            &message(),
            &[recipient(0, "to")],
            &bodies,
            &[],
            AttachmentMode::External,
            true,
            0,
        )
        .unwrap();
        let eml = String::from_utf8(eml).unwrap();
        assert!(eml.contains("X-PSTD-Attachment-Mode: external\r\n"));
        assert!(eml.contains("X-PSTD-Attachment-Manifest: attachments.jsonl\r\n"));
        assert!(eml.contains("X-PSTD-Attachment-Root: .\r\n"));
        assert!(!eml.contains("Content-Transfer-Encoding: base64\r\n"));
        assert!(eml.ends_with("plain body\r\n"));
    }

    #[test]
    fn external_mode_materializes_bytes_and_links_manifest_records() {
        let directory = tempfile::tempdir().unwrap();
        let attachment = attachment(0, b"external bytes");
        let records = vec![attachment.record.clone()];
        let (pending, manifest) = plan_external_attachments(
            Some("message.eml"),
            &records,
            std::slice::from_ref(&attachment),
            &mut BTreeSet::new(),
        )
        .unwrap();
        write_external_attachments(directory.path(), &pending, &manifest).unwrap();

        let path = directory.path().join(&attachment.record.archive_path);
        assert_eq!(fs::read(path).unwrap(), b"external bytes");
        let manifest = fs::read_to_string(directory.path().join("attachments.jsonl")).unwrap();
        assert!(manifest.contains(&attachment.record.attachment_key));
        assert!(manifest.contains("\"materialized_path\":\"attachments/message/"));
        assert!(manifest.contains("\"materialization_status\":\"attachment_file_emitted\""));
    }

    #[test]
    fn external_mode_preserves_empty_and_unavailable_attachment_states() {
        let directory = tempfile::tempdir().unwrap();
        let empty = attachment(0, &[]);
        let unavailable = unavailable_attachment_record(
            "message",
            1,
            Some("missing.bin".to_string()),
            "payload_unavailable",
        );
        let records = vec![empty.record.clone(), unavailable.clone()];
        let (pending, manifest) = plan_external_attachments(
            None,
            &records,
            std::slice::from_ref(&empty),
            &mut BTreeSet::new(),
        )
        .unwrap();
        write_external_attachments(directory.path(), &pending, &manifest).unwrap();

        let empty_path = directory.path().join(&empty.record.archive_path);
        assert!(empty_path.is_file());
        assert!(fs::read(empty_path).unwrap().is_empty());
        let manifest = fs::read_to_string(directory.path().join("attachments.jsonl")).unwrap();
        assert!(manifest.contains("\"materialization_status\":\"attachment_payload_unavailable\""));
        assert!(manifest.contains("\"materialized_path\":null"));
    }

    #[test]
    fn external_mode_preserves_integrity_failure_without_materializing_bytes() {
        let directory = tempfile::tempdir().unwrap();
        let mut invalid = attachment(0, b"external bytes");
        invalid.record.sha256 = "wrong-hash".to_string();
        let records = vec![invalid.record.clone()];
        let (pending, manifest) = plan_external_attachments(
            Some("message.eml"),
            &records,
            std::slice::from_ref(&invalid),
            &mut BTreeSet::new(),
        )
        .unwrap();
        assert!(pending.is_empty());
        write_external_attachments(directory.path(), &pending, &manifest).unwrap();
        assert!(!directory.path().join(&invalid.record.archive_path).exists());
        let manifest = fs::read_to_string(directory.path().join("attachments.jsonl")).unwrap();
        assert!(
            manifest.contains("\"materialization_status\":\"attachment_payload_integrity_failed\"")
        );
        assert!(manifest.contains("\"materialized_path\":null"));
    }

    #[test]
    fn external_mode_rejects_manifest_and_platform_separator_collisions() {
        let mut manifest_collision = attachment(0, b"bytes").record;
        manifest_collision.archive_path = "attachments.jsonl".to_string();
        let mut used_paths = BTreeSet::new();
        used_paths.insert(PathBuf::from("attachments.jsonl"));
        assert!(
            plan_external_attachments(None, &[manifest_collision], &[], &mut used_paths,).is_err()
        );

        let mut platform_separator = attachment(0, b"bytes").record;
        platform_separator.archive_path = "attachments\\\\outside.bin".to_string();
        assert!(relative_attachment_path(&platform_separator).is_err());
    }

    #[test]
    fn rejects_external_attachment_path_escape() {
        let mut record = attachment(0, b"bytes").record;
        record.archive_path = "../outside.bin".to_string();
        assert!(relative_attachment_path(&record).is_err());
    }
}
