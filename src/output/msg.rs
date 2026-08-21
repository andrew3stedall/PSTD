use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

use crate::config::OutputProfile;
use crate::error::PstdResult;
use crate::output::mailbox::{
    folder_path_map, ordered_mail_messages, safe_folder_path, serialize_message_eml,
    MailboxArtifact, MailboxArtifactSummary,
};
use crate::output::metadata::{
    AttachmentRecord, BodyRecord, FolderRecord, HeaderProjectionRecord, MessageRecord,
    RecipientRecord,
};
use crate::pst::attachments::AttachmentPayload;
use crate::pst::item_routing::classify_message_class;
use crate::pst::messages::BodyPayload;

const ENDOFCHAIN: u32 = 0xFFFF_FFFE;
const FREESECT: u32 = 0xFFFF_FFFF;
const FATSECT: u32 = 0xFFFF_FFFD;
const DIFSECT: u32 = 0xFFFF_FFFC;
const MINI_CUTOFF: usize = 4096;
const SECTOR_SIZE: usize = 512;
const MINI_SECTOR_SIZE: usize = 64;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MsgArtifactDecision {
    pub message_key: String,
    pub msg_path: Option<String>,
    pub eml_path: Option<String>,
    pub status: String,
    pub subject: Option<String>,
    pub recipient_count: usize,
    pub body_sha256: Option<String>,
    pub attachment_sha256s: Vec<String>,
    pub unsupported: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MsgProfileStatus {
    pub profile: String,
    pub status: String,
    pub output_root: String,
    pub message_count: usize,
    pub emitted_message_count: usize,
    pub eml_artifact_count: usize,
    pub artifact_count: usize,
    pub skipped_count: usize,
    pub unavailable_count: usize,
    pub attachment_count: usize,
    pub emitted_attachment_count: usize,
    pub unsupported_attachment_count: usize,
    pub path_policy: String,
    pub publication_policy: String,
    pub artifacts: Vec<MailboxArtifactSummary>,
    pub decisions: Vec<MsgArtifactDecision>,
}

#[derive(Debug, Clone)]
pub struct MsgRender {
    pub status: MsgProfileStatus,
    pub artifacts: Vec<MailboxArtifact>,
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
) -> Option<MsgRender> {
    if profile != OutputProfile::Msg {
        return None;
    }

    let ordered = ordered_mail_messages(messages);
    let message_count = ordered.len();
    let folder_paths = folder_path_map(folders, &ordered);
    let mut groups = BTreeMap::<String, Vec<&MessageRecord>>::new();
    let mut decisions = Vec::new();
    let mut skipped_count = 0usize;
    let mut unavailable_count = 0usize;

    for message in ordered {
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
                unavailable_count += 1;
                "unavailable_missing_item_class"
            } else {
                skipped_count += 1;
                "skipped_non_mail_item"
            };
            decisions.push(MsgArtifactDecision {
                message_key: message.message_key.clone(),
                msg_path: None,
                eml_path: None,
                status: status.to_string(),
                subject: message.subject.clone(),
                recipient_count: 0,
                body_sha256: None,
                attachment_sha256s: Vec::new(),
                unsupported: vec![status.to_string()],
            });
            continue;
        }
        let folder = folder_paths
            .get(&(message.folder_key.clone(), message.folder_path.clone()))
            .cloned()
            .unwrap_or_else(|| safe_folder_path(&message.folder_path));
        groups.entry(folder).or_default().push(message);
    }
    for group in groups.values_mut() {
        group.sort_by(|left, right| left.message_key.cmp(&right.message_key));
    }

    let mut artifacts = Vec::new();
    let mut emitted_message_count = 0usize;
    let mut eml_artifact_count = 0usize;
    let mut attachment_count = 0usize;
    let mut emitted_attachment_count = 0usize;
    let mut unsupported_attachment_count = 0usize;

    for (folder, group) in groups {
        for (ordinal, message) in group.iter().enumerate() {
            let base = format!("outputs/msg/{folder}/{}", ordinal + 1);
            let msg_path = format!("{base}.msg");
            let eml_path = format!("{base}.eml");
            let built = build_msg(
                message,
                headers,
                recipients,
                bodies,
                body_payloads,
                attachments,
                attachment_payloads,
            );
            let (msg_bytes, evidence) = match built {
                Ok(value) => value,
                Err(error) => {
                    unavailable_count += 1;
                    decisions.push(MsgArtifactDecision {
                        message_key: message.message_key.clone(),
                        msg_path: None,
                        eml_path: None,
                        status: format!("msg_unavailable: {}", error.status),
                        subject: message.subject.clone(),
                        recipient_count: 0,
                        body_sha256: None,
                        attachment_sha256s: Vec::new(),
                        unsupported: vec![format!("msg_build_error: {}", error.status)],
                    });
                    continue;
                }
            };

            attachment_count += evidence.attachment_count;
            emitted_attachment_count += evidence.emitted_attachment_count;
            unsupported_attachment_count += evidence.unsupported_attachment_count;
            artifacts.push(artifact(
                msg_path.clone(),
                Some(message.message_key.clone()),
                message.folder_path.clone(),
                "msg",
                msg_bytes,
                "msg_ole_emitted",
            ));
            emitted_message_count += 1;

            let eml_result = serialize_message_eml(
                message,
                headers,
                recipients,
                bodies,
                body_payloads,
                attachments,
                attachment_payloads,
                false,
            );
            let mut status = if evidence.unsupported.is_empty() {
                "msg_projection_available".to_string()
            } else {
                "msg_projection_partial".to_string()
            };
            let mut observed_eml_path = None;
            match eml_result {
                Ok(bytes) => {
                    artifacts.push(artifact(
                        eml_path.clone(),
                        Some(message.message_key.clone()),
                        message.folder_path.clone(),
                        "msg_eml",
                        bytes,
                        "msg_eml_compatibility_file_emitted",
                    ));
                    eml_artifact_count += 1;
                    observed_eml_path = Some(eml_path);
                }
                Err(error) => {
                    status = "msg_projection_partial_eml_unavailable".to_string();
                    unavailable_count += 1;
                    decisions.push(MsgArtifactDecision {
                        message_key: message.message_key.clone(),
                        msg_path: Some(msg_path.clone()),
                        eml_path: None,
                        status: status.clone(),
                        subject: message.subject.clone(),
                        recipient_count: evidence.recipient_count,
                        body_sha256: evidence.body_sha256.clone(),
                        attachment_sha256s: evidence.attachment_sha256s.clone(),
                        unsupported: evidence
                            .unsupported
                            .iter()
                            .cloned()
                            .chain(std::iter::once(format!("eml_build_error: {}", error.status)))
                            .collect(),
                    });
                    continue;
                }
            }
            decisions.push(MsgArtifactDecision {
                message_key: message.message_key.clone(),
                msg_path: Some(msg_path),
                eml_path: observed_eml_path,
                status,
                subject: message.subject.clone(),
                recipient_count: evidence.recipient_count,
                body_sha256: evidence.body_sha256,
                attachment_sha256s: evidence.attachment_sha256s,
                unsupported: evidence.unsupported,
            });
        }
    }

    decisions.sort_by(|left, right| left.message_key.cmp(&right.message_key));
    let status = if message_count == 0 || emitted_message_count == 0 {
        "msg_records_unavailable"
    } else if unavailable_count > 0 || skipped_count > 0 || unsupported_attachment_count > 0 {
        "msg_projection_partial"
    } else {
        "msg_projection_available"
    };
    let summaries = artifacts
        .iter()
        .map(|artifact| artifact.summary.clone())
        .collect::<Vec<_>>();
    Some(MsgRender {
        status: MsgProfileStatus {
            profile: "msg".to_string(),
            status: status.to_string(),
            output_root: "outputs/msg".to_string(),
            message_count,
            emitted_message_count,
            eml_artifact_count,
            artifact_count: artifacts.len(),
            skipped_count,
            unavailable_count,
            attachment_count,
            emitted_attachment_count,
            unsupported_attachment_count,
            path_policy: "sanitized relative folder paths; deterministic ordinal filenames".to_string(),
            publication_policy: "complete OLE and compatibility EML bytes are built in memory before TAR publication".to_string(),
            artifacts: summaries,
            decisions,
        },
        artifacts,
    })
}

#[derive(Debug, Default)]
struct MsgEvidence {
    recipient_count: usize,
    attachment_count: usize,
    emitted_attachment_count: usize,
    unsupported_attachment_count: usize,
    body_sha256: Option<String>,
    attachment_sha256s: Vec<String>,
    unsupported: Vec<String>,
}

fn build_msg(
    message: &MessageRecord,
    _headers: &[HeaderProjectionRecord],
    recipients: &[RecipientRecord],
    bodies: &[BodyRecord],
    body_payloads: &[BodyPayload],
    attachments: &[AttachmentRecord],
    attachment_payloads: &[AttachmentPayload],
) -> PstdResult<(Vec<u8>, MsgEvidence)> {
    let mut document = OleDocument::new();
    let root = 0usize;
    let mut evidence = MsgEvidence::default();
    let mut top_props = Vec::new();
    let mut unsupported = BTreeSet::new();

    add_unicode_property(
        &mut document,
        root,
        &mut top_props,
        0x001A,
        message
            .message_class
            .as_deref()
            .unwrap_or("IPM.Note"),
    );
    if let Some(value) = &message.subject {
        add_unicode_property(&mut document, root, &mut top_props, 0x0037, value);
    }
    if let Some(value) = &message.transport_message_headers {
        add_unicode_property(&mut document, root, &mut top_props, 0x007D, value);
    }
    for (property_id, value, label) in [
        (0x1035, message.internet_message_id.as_ref(), "internet_message_id"),
        (0x1042, message.in_reply_to_id.as_ref(), "in_reply_to"),
    ] {
        if let Some(value) = value {
            add_unicode_property(&mut document, root, &mut top_props, property_id, value);
        } else if label == "internet_message_id" && message.has_text_body {
            unsupported.insert("internet_message_id_unavailable".to_string());
        }
    }
    if let Some(value) = &message.sender_name {
        add_unicode_property(&mut document, root, &mut top_props, 0x0042, value);
    }
    if let Some(value) = &message.sender_email {
        add_unicode_property(&mut document, root, &mut top_props, 0x0065, value);
    }
    add_numeric_if_parseable(
        &mut top_props,
        0x0017,
        message.importance.as_deref(),
        "importance",
        &mut unsupported,
    );
    add_numeric_if_parseable(
        &mut top_props,
        0x0026,
        message.priority.as_deref(),
        "priority",
        &mut unsupported,
    );
    add_numeric_if_parseable(
        &mut top_props,
        0x0036,
        message.sensitivity.as_deref(),
        "sensitivity",
        &mut unsupported,
    );
    add_bool_if_parseable(
        &mut top_props,
        0x0023,
        message.delivery_report_requested.as_deref(),
        "delivery_report_requested",
        &mut unsupported,
    );
    add_bool_if_parseable(
        &mut top_props,
        0x0029,
        message.read_receipt_requested.as_deref(),
        "read_receipt_requested",
        &mut unsupported,
    );
    add_bool_if_parseable(
        &mut top_props,
        0x0C17,
        message.reply_requested.as_deref(),
        "reply_requested",
        &mut unsupported,
    );
    add_bool_if_parseable(
        &mut top_props,
        0x0E01,
        message.delete_after_submit.as_deref(),
        "delete_after_submit",
        &mut unsupported,
    );
    add_numeric_if_parseable(
        &mut top_props,
        0x0E07,
        message.message_flags.as_deref(),
        "message_flags",
        &mut unsupported,
    );
    if let Some(value) = &message.sent_at {
        if let Some(filetime) = windows_filetime(value) {
            top_props.push(Property::inline(0x0039, 0x0040, filetime as u32, (filetime >> 32) as u32));
        } else {
            unsupported.insert("sent_at_invalid_or_unavailable".to_string());
        }
    }

    let body_map = body_payloads
        .iter()
        .map(|payload| (payload.record.body_key.as_str(), payload))
        .collect::<BTreeMap<_, _>>();
    let mut body_records = bodies
        .iter()
        .filter(|body| body.message_key == message.message_key)
        .collect::<Vec<_>>();
    body_records.sort_by_key(|body| body.body_key.clone());
    for body in body_records {
        let Some(payload) = body_map.get(body.body_key.as_str()) else {
            unsupported.insert(format!("body_payload_unavailable:{}", body.body_key));
            continue;
        };
        match body.body_type.as_str() {
            "text" => {
                if let Ok(text) = std::str::from_utf8(&payload.bytes) {
                    add_unicode_property(&mut document, root, &mut top_props, 0x1000, text);
                    evidence.body_sha256 = Some(payload.record.sha256.clone());
                } else {
                    unsupported.insert(format!("body_invalid_utf8:{}", body.body_key));
                }
            }
            "html" => {
                if let Ok(text) = std::str::from_utf8(&payload.bytes) {
                    add_unicode_property(&mut document, root, &mut top_props, 0x1013, text);
                } else {
                    unsupported.insert(format!("html_invalid_utf8:{}", body.body_key));
                }
            }
            "rtf" => {
                unsupported.insert("rtf_property_not_represented_raw_evidence_preserved".to_string());
            }
            _ => {
                unsupported.insert(format!("body_type_not_represented:{}", body.body_type));
            }
        }
    }

    let message_recipients = recipients
        .iter()
        .filter(|recipient| recipient.message_key == message.message_key)
        .collect::<Vec<_>>();
    evidence.recipient_count = message_recipients.len();
    for (ordinal, recipient) in message_recipients.iter().enumerate() {
        let storage = document.add_storage(root, format!("__recip_version1.0_#{ordinal:08X}"));
        let mut properties = vec![
            Property::inline(0x0C15, 0x0003, recipient_type(recipient.recipient_type.as_str()), 0),
            Property::inline(0x3000, 0x0003, ordinal as u32, 0),
        ];
        if let Some(value) = recipient.display_name.as_deref().or(recipient.smtp_address.as_deref()) {
            add_unicode_property(&mut document, storage, &mut properties, 0x3001, value);
        }
        if let Some(value) = recipient.address_type.as_deref() {
            add_unicode_property(&mut document, storage, &mut properties, 0x3002, value);
        }
        if let Some(value) = recipient.smtp_address.as_deref().or(recipient.raw_address.as_deref()) {
            add_unicode_property(&mut document, storage, &mut properties, 0x3003, value);
        }
        document.add_stream(storage, "__properties_version1.0", property_stream(&[0; 8], properties));
    }

    let attachment_map = attachment_payloads
        .iter()
        .map(|payload| (payload.record.attachment_key.as_str(), payload))
        .collect::<BTreeMap<_, _>>();
    let message_attachments = attachments
        .iter()
        .filter(|attachment| attachment.message_key == message.message_key)
        .collect::<Vec<_>>();
    evidence.attachment_count = message_attachments.len();
    for (ordinal, attachment) in message_attachments.iter().enumerate() {
        if attachment.attachment_method == Some(5) {
            evidence.unsupported_attachment_count += 1;
            unsupported.insert(format!("embedded_method_5_not_supported:{}", attachment.attachment_key));
            continue;
        }
        let Some(payload) = attachment_map.get(attachment.attachment_key.as_str()) else {
            evidence.unsupported_attachment_count += 1;
            unsupported.insert(format!("attachment_payload_unavailable:{}", attachment.attachment_key));
            continue;
        };
        let storage = document.add_storage(root, format!("__attach_version1.0_#{ordinal:08X}"));
        let mut properties = vec![
            Property::inline(0x0E21, 0x0003, ordinal as u32, 2),
            Property::inline(0x0FF4, 0x0003, 2, 2),
            Property::inline(0x0FF7, 0x0003, 0, 2),
            Property::inline(0x0FFE, 0x0003, 7, 2),
            Property::inline(0x3705, 0x0003, 1, 7),
            Property::inline(0x370B, 0x0003, attachment.rendering_position.unwrap_or(attachment.ordinal) as u32, 7),
            Property::inline(0x3710, 0x0003, attachment.ordinal as u32, 6),
        ];
        add_binary_property(
            &mut document,
            storage,
            &mut properties,
            0x0FF9,
            attachment.attachment_key.as_bytes(),
        );
        add_binary_property(&mut document, storage, &mut properties, 0x3701, &payload.bytes);
        let filename = attachment
            .filename_original
            .as_deref()
            .unwrap_or(attachment.filename_safe.as_str());
        add_unicode_property(&mut document, storage, &mut properties, 0x3707, filename);
        if let Some(value) = attachment.content_type.as_deref() {
            add_unicode_property(&mut document, storage, &mut properties, 0x370E, value);
        }
        document.add_stream(storage, "__properties_version1.0", property_stream(&[0; 8], properties));
        evidence.emitted_attachment_count += 1;
        evidence.attachment_sha256s.push(payload.record.sha256.clone());
        if attachment.attachment_method == Some(6) {
            unsupported.insert(format!("ole_method_6_materialized_by_value:{}", attachment.attachment_key));
        }
    }

    let nameid = document.add_storage(root, "__nameid_version1.0");
    for property_id in [0x0002u16, 0x0003, 0x0004] {
        document.add_stream(nameid, format!("__substg1.0_{property_id:04X}0102"), Vec::new());
    }
    let header = top_property_header(
        message_recipients.len() as u32,
        evidence.emitted_attachment_count as u32,
    );
    document.add_stream(root, "__properties_version1.0", property_stream(&header, top_props));
    evidence.unsupported = unsupported.into_iter().collect();
    Ok((document.finish(), evidence))
}

fn artifact(
    path: String,
    message_key: Option<String>,
    folder_path: String,
    output_kind: &str,
    bytes: Vec<u8>,
    status: &str,
) -> MailboxArtifact {
    let sha256 = sha256(&bytes);
    MailboxArtifact {
        summary: MailboxArtifactSummary {
            path,
            message_key,
            folder_path,
            output_kind: output_kind.to_string(),
            sha256,
            size_bytes: bytes.len() as u64,
            status: status.to_string(),
        },
        bytes,
    }
}

fn recipient_type(value: &str) -> u32 {
    match value.trim().to_ascii_lowercase().as_str() {
        "cc" | "carbon_copy" => 2,
        "bcc" | "blind_carbon_copy" => 3,
        _ => 1,
    }
}

fn add_numeric_if_parseable(
    properties: &mut Vec<Property>,
    property_id: u16,
    value: Option<&str>,
    label: &str,
    unsupported: &mut BTreeSet<String>,
) {
    let Some(value) = value else { return };
    match value.trim().parse::<u32>() {
        Ok(value) => properties.push(Property::inline(property_id, 0x0003, value, 6)),
        Err(_) => {
            unsupported.insert(format!("{label}_not_numeric"));
        }
    }
}

fn add_bool_if_parseable(
    properties: &mut Vec<Property>,
    property_id: u16,
    value: Option<&str>,
    label: &str,
    unsupported: &mut BTreeSet<String>,
) {
    let Some(value) = value else { return };
    let parsed = match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" => Some(1),
        "0" | "false" | "no" => Some(0),
        _ => None,
    };
    match parsed {
        Some(value) => properties.push(Property::inline(property_id, 0x000B, value, 6)),
        None => {
            unsupported.insert(format!("{label}_not_boolean"));
        }
    }
}

fn windows_filetime(value: &str) -> Option<u64> {
    let parsed = DateTime::parse_from_rfc3339(value).ok()?.with_timezone(&Utc);
    let unix_100ns = i128::from(parsed.timestamp()) * 10_000_000
        + i128::from(parsed.timestamp_subsec_nanos() / 100);
    let epoch_offset = i128::from(11_644_473_600i64) * 10_000_000;
    u64::try_from(unix_100ns + epoch_offset).ok()
}

fn add_unicode_property(
    document: &mut OleDocument,
    parent: usize,
    properties: &mut Vec<Property>,
    property_id: u16,
    value: &str,
) {
    let mut bytes = Vec::with_capacity(value.len() * 2 + 2);
    for unit in value.encode_utf16().chain(std::iter::once(0)) {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    document.add_stream(
        parent,
        format!("__substg1.0_{property_id:04X}001F"),
        bytes.clone(),
    );
    properties.push(Property::stream(property_id, 0x001F, bytes.len() as u32));
}

fn add_binary_property(
    document: &mut OleDocument,
    parent: usize,
    properties: &mut Vec<Property>,
    property_id: u16,
    bytes: &[u8],
) {
    document.add_stream(
        parent,
        format!("__substg1.0_{property_id:04X}0102"),
        bytes.to_vec(),
    );
    properties.push(Property::stream(property_id, 0x0102, bytes.len() as u32));
}

fn top_property_header(recipient_count: u32, attachment_count: u32) -> Vec<u8> {
    let mut header = vec![0u8; 32];
    put_u32(&mut header, 8, recipient_count);
    put_u32(&mut header, 12, attachment_count);
    put_u32(&mut header, 16, recipient_count);
    put_u32(&mut header, 20, attachment_count);
    header
}

#[derive(Debug, Clone, Copy)]
struct Property {
    id: u16,
    property_type: u16,
    flags: u32,
    length: u32,
    reserved: u32,
}

impl Property {
    fn inline(id: u16, property_type: u16, length: u32, reserved: u32) -> Self {
        Self {
            id,
            property_type,
            flags: 0x6,
            length,
            reserved,
        }
    }

    fn stream(id: u16, property_type: u16, length: u32) -> Self {
        Self::inline(id, property_type, length, 0)
    }
}

fn property_stream(header: &[u8], mut properties: Vec<Property>) -> Vec<u8> {
    properties.sort_by_key(|property| (property.id, property.property_type));
    let mut output = header.to_vec();
    for property in properties {
        output.extend_from_slice(&property.id.to_le_bytes());
        output.extend_from_slice(&property.property_type.to_le_bytes());
        output.extend_from_slice(&property.flags.to_le_bytes());
        output.extend_from_slice(&property.length.to_le_bytes());
        output.extend_from_slice(&property.reserved.to_le_bytes());
    }
    output
}

fn sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

#[derive(Debug)]
struct OleNode {
    name: String,
    kind: u8,
    children: Vec<usize>,
    child: u32,
    left: u32,
    right: u32,
    data: Vec<u8>,
    start_sector: u32,
    start_mini_sector: u32,
}

#[derive(Debug)]
struct OleDocument {
    nodes: Vec<OleNode>,
}

impl OleDocument {
    fn new() -> Self {
        Self {
            nodes: vec![OleNode {
                name: "Root Entry".to_string(),
                kind: 5,
                children: Vec::new(),
                child: FREESECT,
                left: FREESECT,
                right: FREESECT,
                data: Vec::new(),
                start_sector: ENDOFCHAIN,
                start_mini_sector: ENDOFCHAIN,
            }],
        }
    }

    fn add_storage(&mut self, parent: usize, name: impl Into<String>) -> usize {
        let index = self.nodes.len();
        self.nodes.push(OleNode {
            name: sanitize_ole_name(&name.into()),
            kind: 1,
            children: Vec::new(),
            child: FREESECT,
            left: FREESECT,
            right: FREESECT,
            data: Vec::new(),
            start_sector: ENDOFCHAIN,
            start_mini_sector: ENDOFCHAIN,
        });
        self.nodes[parent].children.push(index);
        index
    }

    fn add_stream(&mut self, parent: usize, name: impl Into<String>, data: Vec<u8>) -> usize {
        let index = self.nodes.len();
        self.nodes.push(OleNode {
            name: sanitize_ole_name(&name.into()),
            kind: 2,
            children: Vec::new(),
            child: FREESECT,
            left: FREESECT,
            right: FREESECT,
            data,
            start_sector: ENDOFCHAIN,
            start_mini_sector: ENDOFCHAIN,
        });
        self.nodes[parent].children.push(index);
        index
    }

    fn finish(mut self) -> Vec<u8> {
        for parent in 0..self.nodes.len() {
            let children = self.nodes[parent].children.clone();
            self.nodes[parent].child = build_directory_tree(&mut self.nodes, &children);
        }

        let mut mini_fat = Vec::<u32>::new();
        let mut mini_stream = Vec::new();
        for node in self.nodes.iter_mut().skip(1) {
            if node.kind == 2 && !node.data.is_empty() && node.data.len() < MINI_CUTOFF {
                node.start_mini_sector = (mini_fat.len()) as u32;
                let mini_count = sectors(node.data.len(), MINI_SECTOR_SIZE);
                for index in 0..mini_count {
                    let mini_id = mini_fat.len() as u32;
                    mini_fat.push(if index + 1 == mini_count {
                        ENDOFCHAIN
                    } else {
                        mini_id + 1
                    });
                    let start = index * MINI_SECTOR_SIZE;
                    let end = (start + MINI_SECTOR_SIZE).min(node.data.len());
                    mini_stream.extend_from_slice(&node.data[start..end]);
                    mini_stream.resize(mini_stream.len() + MINI_SECTOR_SIZE - (end - start), 0);
                }
            }
        }

        let mut next_sector = 0usize;
        for node in self.nodes.iter_mut().skip(1) {
            if node.kind == 2 && node.data.len() >= MINI_CUTOFF {
                node.start_sector = next_sector as u32;
                next_sector += sectors(node.data.len(), SECTOR_SIZE);
            }
        }
        let root_mini_stream_start = if mini_stream.is_empty() {
            ENDOFCHAIN
        } else {
            let start = next_sector as u32;
            next_sector += sectors(mini_stream.len(), SECTOR_SIZE);
            start
        };
        let directory_sector_count = sectors(self.nodes.len() * 128, SECTOR_SIZE);
        let directory_start = next_sector as u32;
        next_sector += directory_sector_count;
        let mini_fat_sector_count = if mini_fat.is_empty() {
            0
        } else {
            sectors(mini_fat.len() * 4, SECTOR_SIZE)
        };
        let mini_fat_start = if mini_fat_sector_count == 0 {
            ENDOFCHAIN
        } else {
            let start = next_sector as u32;
            next_sector += mini_fat_sector_count;
            start
        };

        self.nodes[0].start_sector = root_mini_stream_start;

        let base_sectors = next_sector;
        let (fat_sector_count, difat_sector_count) = fat_layout(base_sectors);
        let fat_start = next_sector as u32;
        next_sector += fat_sector_count;
        let difat_start = if difat_sector_count == 0 {
            ENDOFCHAIN
        } else {
            let start = next_sector as u32;
            next_sector += difat_sector_count;
            start
        };
        let total_sector_count = next_sector;
        let mut sectors_data = vec![0u8; total_sector_count * SECTOR_SIZE];
        let mut fat = vec![FREESECT; total_sector_count];

        for node in self.nodes.iter() {
            if node.kind == 2 && node.data.len() >= MINI_CUTOFF {
                write_sector_chain(
                    &mut fat,
                    node.start_sector as usize,
                    sectors(node.data.len(), SECTOR_SIZE),
                );
                write_bytes_at_sectors(
                    &mut sectors_data,
                    node.start_sector as usize,
                    &node.data,
                );
            }
        }
        if !mini_stream.is_empty() {
            write_sector_chain(
                &mut fat,
                root_mini_stream_start as usize,
                sectors(mini_stream.len(), SECTOR_SIZE),
            );
            write_bytes_at_sectors(
                &mut sectors_data,
                root_mini_stream_start as usize,
                &mini_stream,
            );
        }
        write_sector_chain(&mut fat, directory_start as usize, directory_sector_count);
        write_bytes_at_sectors(
            &mut sectors_data,
            directory_start as usize,
            &directory_bytes(&self.nodes, mini_stream.len()),
        );
        if mini_fat_sector_count > 0 {
            write_sector_chain(&mut fat, mini_fat_start as usize, mini_fat_sector_count);
            let mut bytes = Vec::with_capacity(mini_fat.len() * 4);
            for value in mini_fat {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
            write_bytes_at_sectors(&mut sectors_data, mini_fat_start as usize, &bytes);
        }
        for index in 0..fat_sector_count {
            fat[fat_start as usize + index] = FATSECT;
        }
        for index in 0..difat_sector_count {
            fat[difat_start as usize + index] = DIFSECT;
        }
        for (index, value) in fat.iter().enumerate() {
            let offset = index * 4;
            let sector = offset / SECTOR_SIZE;
            let within = offset % SECTOR_SIZE;
            if sector < fat_sector_count {
                let target = (fat_start as usize + sector) * SECTOR_SIZE + within;
                sectors_data[target..target + 4].copy_from_slice(&value.to_le_bytes());
            }
        }
        if difat_sector_count > 0 {
            write_difat(&mut sectors_data, difat_start as usize, difat_sector_count, fat_start, fat_sector_count);
        }

        let mut header = vec![0u8; SECTOR_SIZE];
        header[0..8].copy_from_slice(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]);
        put_u16(&mut header, 24, 0x003E);
        put_u16(&mut header, 26, 3);
        put_u16(&mut header, 28, 0xFFFE);
        put_u16(&mut header, 30, 9);
        put_u16(&mut header, 32, 6);
        put_u32(&mut header, 40, 0);
        put_u32(&mut header, 44, fat_sector_count as u32);
        put_u32(&mut header, 48, directory_start);
        put_u32(&mut header, 52, 0);
        put_u32(&mut header, 56, MINI_CUTOFF as u32);
        put_u32(&mut header, 60, mini_fat_start);
        put_u32(&mut header, 64, mini_fat_sector_count as u32);
        put_u32(&mut header, 68, difat_start);
        put_u32(&mut header, 72, difat_sector_count as u32);
        let header_difat_count = fat_sector_count.min(109);
        for index in 0..109 {
            let value = if index < header_difat_count {
                fat_start + index as u32
            } else {
                FREESECT
            };
            put_u32(&mut header, 76 + index * 4, value);
        }

        let mut output = header;
        output.extend_from_slice(&sectors_data);
        output
    }
}

fn sanitize_ole_name(name: &str) -> String {
    let mut units = name.encode_utf16().collect::<Vec<_>>();
    units.truncate(31);
    String::from_utf16(&units).unwrap_or_else(|_| "_".to_string())
}

fn build_directory_tree(nodes: &mut [OleNode], children: &[usize]) -> u32 {
    if children.is_empty() {
        return FREESECT;
    }
    let mut ordered = children.to_vec();
    ordered.sort_by(|left, right| {
        nodes[*left]
            .name
            .to_ascii_lowercase()
            .cmp(&nodes[*right].name.to_ascii_lowercase())
            .then(nodes[*left].name.cmp(&nodes[*right].name))
    });
    let middle = ordered.len() / 2;
    let root = ordered[middle];
    let left = build_directory_tree(nodes, &ordered[..middle]);
    let right = build_directory_tree(nodes, &ordered[middle + 1..]);
    nodes[root].left = left;
    nodes[root].right = right;
    root as u32
}

fn directory_bytes(nodes: &[OleNode], mini_stream_size: usize) -> Vec<u8> {
    let mut output = vec![0u8; sectors(nodes.len() * 128, SECTOR_SIZE) * SECTOR_SIZE];
    for (index, node) in nodes.iter().enumerate() {
        let offset = index * 128;
        let mut name = node.name.encode_utf16().collect::<Vec<_>>();
        name.truncate(31);
        for (unit_index, unit) in name.iter().enumerate() {
            put_u16(&mut output, offset + unit_index * 2, *unit);
        }
        put_u16(&mut output, offset + 64, ((name.len() + 1) * 2) as u16);
        output[offset + 66] = node.kind;
        output[offset + 67] = 1;
        put_u32(&mut output, offset + 68, node.left);
        put_u32(&mut output, offset + 72, node.right);
        put_u32(&mut output, offset + 76, node.child);
        let start = if index == 0 {
            node.start_sector
        } else if node.kind == 2 && !node.data.is_empty() && node.data.len() < MINI_CUTOFF {
            node.start_mini_sector
        } else {
            node.start_sector
        };
        put_u32(&mut output, offset + 116, start);
        let size = if index == 0 { mini_stream_size } else { node.data.len() };
        put_u64(&mut output, offset + 120, size as u64);
    }
    output
}

fn write_difat(bytes: &mut [u8], start: usize, count: usize, fat_start: u32, fat_count: usize) {
    for index in 0..count {
        let offset = (start + index) * SECTOR_SIZE;
        for slot in 0..127 {
            let fat_index = 109 + index * 127 + slot;
            let value = if fat_index < fat_count {
                fat_start + fat_index as u32
            } else {
                FREESECT
            };
            put_u32(bytes, offset + slot * 4, value);
        }
        let next = if index + 1 == count {
            ENDOFCHAIN
        } else {
            (start + index + 1) as u32
        };
        put_u32(bytes, offset + 127 * 4, next);
    }
}

fn fat_layout(base_sectors: usize) -> (usize, usize) {
    let mut fat_count = 1usize;
    let mut difat_count = 0usize;
    loop {
        let next_fat_count = (base_sectors + difat_count).div_ceil(127).max(1);
        let next_difat_count = next_fat_count.saturating_sub(109).div_ceil(127);
        if next_fat_count == fat_count && next_difat_count == difat_count {
            return (fat_count, difat_count);
        }
        fat_count = next_fat_count;
        difat_count = next_difat_count;
    }
}

fn sectors(length: usize, sector_size: usize) -> usize {
    length.div_ceil(sector_size)
}

fn write_sector_chain(fat: &mut [u32], start: usize, count: usize) {
    for index in 0..count {
        fat[start + index] = if index + 1 == count {
            ENDOFCHAIN
        } else {
            (start + index + 1) as u32
        };
    }
}

fn write_bytes_at_sectors(output: &mut [u8], start_sector: usize, bytes: &[u8]) {
    let offset = start_sector * SECTOR_SIZE;
    output[offset..offset + bytes.len()].copy_from_slice(bytes);
}

fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn put_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_a_valid_ole_header_and_deterministic_bytes() {
        let mut first = OleDocument::new();
        let storage = first.add_storage(0, "__nameid_version1.0");
        first.add_stream(storage, "__substg1.0_00020102", Vec::new());
        first.add_stream(0, "__properties_version1.0", property_stream(&[0; 32], vec![Property::inline(0x001A, 0x0003, 1, 0)]));
        let first = first.finish();
        let mut second = OleDocument::new();
        let storage = second.add_storage(0, "__nameid_version1.0");
        second.add_stream(storage, "__substg1.0_00020102", Vec::new());
        second.add_stream(0, "__properties_version1.0", property_stream(&[0; 32], vec![Property::inline(0x001A, 0x0003, 1, 0)]));
        let second = second.finish();
        assert_eq!(&first[..8], &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]);
        assert_eq!(first, second);
    }

    #[test]
    fn invalid_dates_are_explicit_and_valid_dates_use_filetime_epoch() {
        assert_eq!(windows_filetime("not-a-date"), None);
        assert_eq!(windows_filetime("1601-01-01T00:00:00Z"), Some(0));
    }
}
