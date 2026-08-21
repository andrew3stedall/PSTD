use std::collections::{BTreeMap, BTreeSet};

use sha2::{Digest, Sha256};

use crate::config::OutputProfile;
use crate::output::ids;
use crate::output::headers::{
    clean_header_value, encode_display_name, encode_mime_parameter, encode_unstructured_value,
};
use crate::output::metadata::{
    AttachmentRecord, HeaderProjectionRecord, MessageRecord, RecipientRecord,
};
use crate::output::paths::{sanitize_segment, UniquePathTracker};
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
    pub attachment_count: usize,
    pub emitted_attachment_count: usize,
    pub filtered_attachment_count: usize,
    pub attachment_unavailable_count: usize,
    pub attachment_filter: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kmail_index_policy: Option<String>,
    pub path_policy: String,
    pub publication_policy: String,
    pub artifacts: Vec<MailboxArtifactSummary>,
    pub decisions: Vec<MailboxDecision>,
    pub attachment_decisions: Vec<MailboxAttachmentDecision>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MailboxDecision {
    pub message_key: String,
    pub folder_path: String,
    pub status: String,
    pub source_class: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MailboxAttachmentDecision {
    pub attachment_key: String,
    pub message_key: String,
    pub filename: String,
    pub path: Option<String>,
    pub status: String,
    pub extension: Option<String>,
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
    Kmail,
    Thunderbird,
}

impl MailMode {
    fn from_profile(profile: OutputProfile) -> Option<Self> {
        match profile {
            OutputProfile::Mbox => Some(Self::Mbox),
            OutputProfile::RecursiveMbox => Some(Self::RecursiveMbox),
            OutputProfile::Mh => Some(Self::Mh),
            OutputProfile::Eml => Some(Self::Eml),
            OutputProfile::Separate => Some(Self::Separate),
            OutputProfile::Kmail => Some(Self::Kmail),
            OutputProfile::Thunderbird => Some(Self::Thunderbird),
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
            Self::Kmail => "kmail",
            Self::Thunderbird => "thunderbird",
        }
    }

    fn output_root(self) -> &'static str {
        match self {
            Self::Mbox => "outputs/mbox",
            Self::RecursiveMbox => "outputs/recursive_mbox",
            Self::Mh => "outputs/mh",
            Self::Eml => "outputs/eml",
            Self::Separate => "outputs/separate",
            Self::Kmail => "outputs/kmail",
            Self::Thunderbird => "outputs/thunderbird",
        }
    }
}

#[derive(Debug)]
pub(crate) struct RenderError {
    pub(crate) status: &'static str,
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
    attachment_extensions: &[String],
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
    let mut attachment_count = 0usize;
    let mut emitted_attachment_count = 0usize;
    let mut filtered_attachment_count = 0usize;
    let mut attachment_unavailable_count = 0usize;
    let mut attachment_decisions = Vec::new();
    let mut attachment_paths = UniquePathTracker::default();

    if mode != MailMode::Separate {
        for attachment in attachments.iter().filter(|attachment| {
            eligible_messages_contain(attachment.message_key.as_str(), &eligible)
        }) {
            attachment_count += 1;
            if !attachment_extension_allowed(attachment, attachment_extensions) {
                filtered_attachment_count += 1;
                attachment_decisions.push(MailboxAttachmentDecision {
                    attachment_key: attachment.attachment_key.clone(),
                    message_key: attachment.message_key.clone(),
                    filename: sanitize_segment(&attachment.filename_safe),
                    path: None,
                    status: "filtered_attachment_extension".to_string(),
                    extension: attachment.extension.clone(),
                });
            }
        }
    }

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
            MailMode::Mbox | MailMode::RecursiveMbox | MailMode::Kmail | MailMode::Thunderbird => {
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
                        attachment_extensions,
                        false,
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
                    let path = match mode {
                        MailMode::Mbox => format!("{}/{}/email.mbox", mode.output_root(), folder),
                        MailMode::RecursiveMbox => {
                            format!("{}/{}/mbox", mode.output_root(), folder)
                        }
                        MailMode::Kmail => kmail_mbox_path(folder),
                        MailMode::Thunderbird => {
                            format!("{}/{}/mbox", mode.output_root(), folder)
                        }
                        _ => unreachable!(),
                    };
                    artifacts.push(artifact(
                        path,
                        None,
                        group[0].folder_path.clone(),
                        mode.name(),
                        output,
                        "mailbox_stream_emitted",
                    ));
                    if mode == MailMode::Thunderbird {
                        append_thunderbird_sidecars(folder, group, folders, &mut artifacts);
                    }
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
                        attachment_extensions,
                        mode == MailMode::Separate,
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
                                path.clone(),
                                Some(message.message_key.clone()),
                                message.folder_path.clone(),
                                mode.name(),
                                bytes,
                                "message_file_emitted",
                            ));
                            emitted_message_count += 1;
                            if mode == MailMode::Separate {
                                append_separate_attachments(
                                    message,
                                    &format!("{}/{folder}", mode.output_root()),
                                    &filename,
                                    attachment_extensions,
                                    attachments,
                                    attachment_payloads,
                                    &mut artifacts,
                                    &mut attachment_decisions,
                                    &mut attachment_paths,
                                    &mut attachment_count,
                                    &mut emitted_attachment_count,
                                    &mut filtered_attachment_count,
                                    &mut attachment_unavailable_count,
                                );
                            }
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
    attachment_decisions.sort_by(|left, right| {
        left.message_key
            .cmp(&right.message_key)
            .then(left.attachment_key.cmp(&right.attachment_key))
            .then(left.status.cmp(&right.status))
    });
    let status = if message_count == 0 || emitted_message_count == 0 {
        "mail_records_unavailable"
    } else if unavailable_count > 0
        || skipped_count > 0
        || filtered_attachment_count > 0
        || attachment_unavailable_count > 0
    {
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
            attachment_count,
            emitted_attachment_count,
            filtered_attachment_count,
            attachment_unavailable_count,
            attachment_filter: if attachment_extensions.is_empty() {
                "all extensions; unnamed and extensionless attachments allowed".to_string()
            } else {
                format!("allowlist: {}", attachment_extensions.join(","))
            },
            kmail_index_policy: (mode == MailMode::Kmail).then(|| {
                "existing ../.<folder>.index is logically invalidated; no mutable index is emitted"
                    .to_string()
            }),
            path_policy: "sanitized relative paths; folder collisions receive stable suffixes".to_string(),
            publication_policy: "archive entries are appended only after complete in-memory serialization; no partial direct files".to_string(),
            artifacts: summaries,
            decisions,
            attachment_decisions,
        },
        artifacts,
    })
}

pub(crate) fn ordered_mail_messages(messages: &[MessageRecord]) -> Vec<&MessageRecord> {
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

pub(crate) fn folder_path_map(
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

pub(crate) fn safe_folder_path(folder_path: &str) -> String {
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

fn kmail_mbox_path(folder: &str) -> String {
    let mut segments = folder
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(sanitize_segment)
        .collect::<Vec<_>>();
    let leaf = segments.pop().unwrap_or_else(|| "root".to_string());
    let parent = if segments.is_empty() {
        String::new()
    } else {
        format!("{}/", segments.join("/"))
    };
    format!("outputs/kmail/{parent}.{leaf}.directory/{leaf}.mbox")
}

fn append_thunderbird_sidecars(
    folder: &str,
    group: &[&MessageRecord],
    folders: &[FolderRecord],
    artifacts: &mut Vec<MailboxArtifact>,
) {
    let folder_record = folders
        .iter()
        .find(|record| record.folder_path == group[0].folder_path);
    let folder_key = group[0].folder_key.clone();
    let folder_path = group[0].folder_path.clone();
    let stored_count = folder_record
        .and_then(|record| record.item_count_total)
        .unwrap_or(group.len() as u64);
    let type_bytes = serde_json::to_vec_pretty(&serde_json::json!({
        "folder_key": folder_key,
        "folder_path": folder_path,
        "readpst_type": serde_json::Value::Null,
        "status": "folder_item_type_unavailable_no_guess",
        "source": "canonical_folder_record",
    }))
    .unwrap_or_else(|_| b"{}".to_vec());
    artifacts.push(artifact(
        format!("outputs/thunderbird/{folder}/.type"),
        None,
        group[0].folder_path.clone(),
        "thunderbird_sidecar",
        type_bytes,
        "sidecar_type_explicit_unavailable",
    ));
    let size_bytes = format!("{} {stored_count}\n", group.len()).into_bytes();
    artifacts.push(artifact(
        format!("outputs/thunderbird/{folder}/.size"),
        None,
        group[0].folder_path.clone(),
        "thunderbird_sidecar",
        size_bytes,
        "sidecar_size_emitted",
    ));
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

fn append_separate_attachments(
    message: &MessageRecord,
    folder_root: &str,
    message_filename: &str,
    attachment_extensions: &[String],
    attachments: &[AttachmentRecord],
    attachment_payloads: &[AttachmentPayload],
    artifacts: &mut Vec<MailboxArtifact>,
    decisions: &mut Vec<MailboxAttachmentDecision>,
    paths: &mut UniquePathTracker,
    attachment_count: &mut usize,
    emitted_attachment_count: &mut usize,
    filtered_attachment_count: &mut usize,
    attachment_unavailable_count: &mut usize,
) {
    let mut selected = attachments
        .iter()
        .filter(|attachment| attachment.message_key == message.message_key)
        .collect::<Vec<_>>();
    selected.sort_by(|left, right| {
        left.ordinal
            .cmp(&right.ordinal)
            .then(left.attachment_key.cmp(&right.attachment_key))
    });

    for attachment in selected {
        *attachment_count += 1;
        let filename = sanitize_segment(&attachment.filename_safe);
        if !attachment_extension_allowed(attachment, attachment_extensions) {
            *filtered_attachment_count += 1;
            decisions.push(MailboxAttachmentDecision {
                attachment_key: attachment.attachment_key.clone(),
                message_key: message.message_key.clone(),
                filename,
                path: None,
                status: "filtered_attachment_extension".to_string(),
                extension: attachment.extension.clone(),
            });
            continue;
        }
        if attachment.attachment_method
            == Some(crate::pst::attachments::ATTACH_METHOD_EMBEDDED_MESSAGE)
        {
            *attachment_unavailable_count += 1;
            decisions.push(MailboxAttachmentDecision {
                attachment_key: attachment.attachment_key.clone(),
                message_key: message.message_key.clone(),
                filename,
                path: None,
                status: "skipped_embedded_message_attachment".to_string(),
                extension: attachment.extension.clone(),
            });
            continue;
        }
        let Some(payload) = attachment_payloads
            .iter()
            .find(|payload| payload.record.attachment_key == attachment.attachment_key)
        else {
            *attachment_unavailable_count += 1;
            decisions.push(MailboxAttachmentDecision {
                attachment_key: attachment.attachment_key.clone(),
                message_key: message.message_key.clone(),
                filename,
                path: None,
                status: "unavailable_attachment_payload".to_string(),
                extension: attachment.extension.clone(),
            });
            continue;
        };
        if payload.bytes.is_empty() {
            *attachment_unavailable_count += 1;
            decisions.push(MailboxAttachmentDecision {
                attachment_key: attachment.attachment_key.clone(),
                message_key: message.message_key.clone(),
                filename,
                path: None,
                status: "unavailable_zero_length_attachment".to_string(),
                extension: attachment.extension.clone(),
            });
            continue;
        }

        let base_name = format!("{message_filename}-{filename}");
        let safe_name = paths.unique_file_name(&base_name);
        let path = format!("{folder_root}/{safe_name}");
        artifacts.push(artifact(
            path.clone(),
            Some(message.message_key.clone()),
            message.folder_path.clone(),
            "attachment_file",
            payload.bytes.clone(),
            "attachment_file_emitted",
        ));
        *emitted_attachment_count += 1;
        decisions.push(MailboxAttachmentDecision {
            attachment_key: attachment.attachment_key.clone(),
            message_key: message.message_key.clone(),
            filename,
            path: Some(path),
            status: "attachment_file_emitted".to_string(),
            extension: attachment.extension.clone(),
        });
    }
}

pub(crate) fn serialize_message_eml(
    message: &MessageRecord,
    headers: &[HeaderProjectionRecord],
    recipients: &[RecipientRecord],
    bodies: &[BodyRecord],
    body_payloads: &[BodyPayload],
    attachments: &[AttachmentRecord],
    attachment_payloads: &[AttachmentPayload],
    attachment_extensions: &[String],
    separate_attachments: bool,
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
        let subject = message
            .subject
            .as_deref()
            .and_then(clean_header)
            .map(|value| encode_unstructured_value(&value))
            .unwrap_or_default();
        push_header(&mut output, "Subject", &subject);
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
        .filter(|attachment| {
            attachment.message_key == message.message_key
                && attachment_extension_allowed(attachment, attachment_extensions)
        })
        .collect::<Vec<_>>()
    {
        if separate_attachments {
            continue;
        }
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
        &format!(
            "{}; {}",
            safe_media_type,
            encode_mime_parameter("name", filename)
        ),
    );
    push_header(output, "Content-Transfer-Encoding", "base64");
    push_header(
        output,
        "Content-Disposition",
        &format!(
            "{}; {}",
            if inline { "inline" } else { "attachment" },
            encode_mime_parameter("filename", filename)
        ),
    );
    if let Some(content_id) = content_id.and_then(clean_header) {
        push_header(output, "Content-ID", &format!("<{}>", content_id));
    }
    output.extend_from_slice(b"\r\n");
    output.extend_from_slice(base64_encode(bytes).as_bytes());
    output.extend_from_slice(b"\r\n");
}

fn eligible_messages_contain(message_key: &str, eligible: &[&MessageRecord]) -> bool {
    eligible
        .iter()
        .any(|message| message.message_key == message_key)
}

pub(crate) fn attachment_extension_allowed(
    attachment: &AttachmentRecord,
    attachment_extensions: &[String],
) -> bool {
    attachment_extensions.is_empty()
        || attachment
            .extension
            .as_ref()
            .is_none_or(|extension| attachment_extensions.contains(&extension.to_ascii_lowercase()))
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
    clean_header_value(value)
}

fn format_address(name: Option<&str>, address: &str) -> String {
    match name.and_then(clean_header) {
        Some(name) => format!("{} <{}>", encode_display_name(&name), address),
        None => address.to_string(),
    }
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
    use crate::pst::attachments::{
        attachment_payload, unavailable_attachment_record, AttachmentMetadata,
    };
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
            &[],
        )
        .expect("eml profile");
        let eml = String::from_utf8(output.artifacts[0].bytes.clone()).expect("utf8 eml");
        assert_eq!(eml.matches("MIME-Version:").count(), 1);
        assert_eq!(eml.matches("Content-Type:").count(), 2);
        assert!(eml.contains("X-Test: yes\r\n"));
    }

    #[test]
    fn separate_profile_emits_binary_attachments_with_filter_decisions() {
        let msg = message("msg-attachments", "/Inbox", Some("IPM.Note"));
        let body = text_body_payload("msg-attachments", "body");
        let payload = attachment_payload(
            "msg-attachments",
            1,
            AttachmentMetadata {
                filename_original: Some("report.PDF".to_string()),
                content_type: Some("application/pdf".to_string()),
                ..AttachmentMetadata::default()
            },
            b"pdf-bytes".to_vec(),
        );
        let filtered = attachment_payload(
            "msg-attachments",
            2,
            AttachmentMetadata {
                filename_original: Some("image.png".to_string()),
                content_type: Some("image/png".to_string()),
                ..AttachmentMetadata::default()
            },
            b"png-bytes".to_vec(),
        );
        let unavailable = unavailable_attachment_record(
            "msg-attachments",
            3,
            Some("missing.pdf".to_string()),
            "payload_unavailable",
        );
        let attachments = vec![
            payload.record.clone(),
            filtered.record.clone(),
            unavailable.clone(),
        ];
        let output = render_profile(
            OutputProfile::Separate,
            &[],
            &[msg],
            &[],
            &[],
            std::slice::from_ref(&body.record),
            std::slice::from_ref(&body),
            &attachments,
            &[payload.clone(), filtered],
            &["pdf".to_string()],
        )
        .expect("separate profile");
        assert_eq!(output.status.emitted_attachment_count, 1);
        assert_eq!(output.status.filtered_attachment_count, 1);
        assert_eq!(output.status.attachment_unavailable_count, 1);
        assert_eq!(output.artifacts.len(), 2);
        assert!(output
            .artifacts
            .iter()
            .any(|artifact| artifact.summary.path.ends_with("1-report.PDF")));
        assert!(output.status.attachment_decisions.iter().any(|decision| {
            decision.status == "filtered_attachment_extension" && decision.filename == "image.png"
        }));
        assert!(output.status.attachment_decisions.iter().any(|decision| {
            decision.status == "unavailable_attachment_payload"
                && decision.filename == "missing.pdf"
        }));
    }

    #[test]
    fn all_mail_profiles_apply_attachment_extension_filters() {
        let msg = message("msg-filtered-eml", "/Inbox", Some("IPM.Note"));
        let body = text_body_payload("msg-filtered-eml", "body");
        let pdf = attachment_payload(
            "msg-filtered-eml",
            1,
            AttachmentMetadata {
                filename_original: Some("report.pdf".to_string()),
                content_type: Some("application/pdf".to_string()),
                ..AttachmentMetadata::default()
            },
            b"pdf-bytes".to_vec(),
        );
        let image = attachment_payload(
            "msg-filtered-eml",
            2,
            AttachmentMetadata {
                filename_original: Some("image.png".to_string()),
                content_type: Some("image/png".to_string()),
                ..AttachmentMetadata::default()
            },
            b"png-bytes".to_vec(),
        );
        let output = render_profile(
            OutputProfile::Eml,
            &[],
            &[msg],
            &[],
            &[],
            std::slice::from_ref(&body.record),
            std::slice::from_ref(&body),
            &[pdf.record.clone(), image.record.clone()],
            &[pdf, image],
            &["PDF".to_ascii_lowercase()],
        )
        .expect("eml profile");
        let eml = String::from_utf8(output.artifacts[0].bytes.clone()).expect("utf8 eml");
        assert!(eml.contains("filename=\"report.pdf\""));
        assert!(!eml.contains("filename=\"image.png\""));
        assert_eq!(output.status.filtered_attachment_count, 1);
        assert!(output.status.attachment_decisions.iter().any(|decision| {
            decision.filename == "image.png" && decision.status == "filtered_attachment_extension"
        }));
    }

    #[test]
    fn generated_eml_uses_rfc2047_and_rfc2231_for_non_ascii_values() {
        let mut msg = message("msg-encoded-headers", "/Inbox", Some("IPM.Note"));
        msg.subject = Some("Résumé — réunion".to_string());
        msg.sender_name = Some("Zoë Example".to_string());
        let body = text_body_payload("msg-encoded-headers", "body");
        let attachment = attachment_payload(
            "msg-encoded-headers",
            1,
            AttachmentMetadata {
                filename_original: Some("résumé final.pdf".to_string()),
                content_type: Some("application/pdf".to_string()),
                ..AttachmentMetadata::default()
            },
            b"pdf-bytes".to_vec(),
        );
        let output = render_profile(
            OutputProfile::Eml,
            &[],
            &[msg],
            &[],
            &[],
            std::slice::from_ref(&body.record),
            std::slice::from_ref(&body),
            std::slice::from_ref(&attachment.record),
            std::slice::from_ref(&attachment),
            &[],
        )
        .expect("eml profile");
        let eml = String::from_utf8(output.artifacts[0].bytes.clone()).expect("utf8 eml");
        assert!(eml.contains("Subject: =?UTF-8?B?UsOpc3Vtw6kg4oCUIHLDqXVuaW9u?=\r\n"));
        assert!(eml.contains("From: =?UTF-8?B?Wm/DqyBFeGFtcGxl?= <sender@example.test>\r\n"));
        assert!(eml.contains("filename*=UTF-8''r%C3%A9sum%C3%A9%20final.pdf"));
    }

    #[test]
    fn kmail_profile_uses_directory_mbox_and_explicit_index_policy() {
        let msg = message("msg-kmail", "/Inbox", Some("IPM.Note"));
        let body = text_body_payload("msg-kmail", "body");
        let output = render_profile(
            OutputProfile::Kmail,
            &[],
            &[msg],
            &[],
            &[],
            std::slice::from_ref(&body.record),
            std::slice::from_ref(&body),
            &[],
            &[],
            &[],
        )
        .expect("kmail profile");
        assert!(output.artifacts[0]
            .summary
            .path
            .ends_with(".Inbox.directory/Inbox.mbox"));
        assert!(output.status.kmail_index_policy.is_some());
    }

    #[test]
    fn thunderbird_profile_emits_mbox_and_explicit_sidecars() {
        let msg = message("msg-thunderbird", "/Inbox", Some("IPM.Note"));
        let body = text_body_payload("msg-thunderbird", "body");
        let output = render_profile(
            OutputProfile::Thunderbird,
            &[],
            &[msg],
            &[],
            &[],
            std::slice::from_ref(&body.record),
            std::slice::from_ref(&body),
            &[],
            &[],
            &[],
        )
        .expect("thunderbird profile");
        assert!(output
            .artifacts
            .iter()
            .any(|artifact| artifact.summary.path.ends_with("/mbox")));
        assert!(output
            .artifacts
            .iter()
            .any(|artifact| artifact.summary.path.ends_with("/.type")));
        assert!(output
            .artifacts
            .iter()
            .any(|artifact| artifact.summary.path.ends_with("/.size")));
        assert!(String::from_utf8_lossy(
            &output
                .artifacts
                .iter()
                .find(|artifact| artifact.summary.path.ends_with("/.type"))
                .expect("type sidecar")
                .bytes
        )
        .contains("folder_item_type_unavailable_no_guess"));
    }

    #[allow(dead_code)]
    fn _body_record(_: BodyRecord) {}
}
