use sha2::{Digest, Sha256};

use crate::output::ids;
use crate::output::metadata::AttachmentRecord;

#[derive(Debug, Clone)]
pub struct AttachmentPayload {
    pub record: AttachmentRecord,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct AttachmentMetadata {
    pub filename_original: Option<String>,
    pub content_type: Option<String>,
    pub is_inline: bool,
    pub content_id: Option<String>,
}

pub fn attachment_payload(
    message_key: &str,
    ordinal: usize,
    metadata: AttachmentMetadata,
    bytes: Vec<u8>,
) -> AttachmentPayload {
    let attachment_key = ids::attachment_key(message_key, ordinal);
    let filename_safe = safe_filename(metadata.filename_original.as_deref(), ordinal);
    let extension = file_extension(&filename_safe);
    let archive_path = format!("attachments/{message_key}/{attachment_key}_{filename_safe}");
    let sha256 = sha256_hex(&bytes);

    AttachmentPayload {
        record: AttachmentRecord {
            message_key: message_key.to_string(),
            attachment_key,
            filename_original: metadata.filename_original,
            filename_safe,
            content_type: metadata.content_type,
            extension,
            size_bytes: bytes.len() as u64,
            sha256,
            is_inline: metadata.is_inline,
            content_id: metadata.content_id,
            ordinal: ordinal as u64,
            archive_path,
            extraction_status: "extracted".to_string(),
        },
        bytes,
    }
}

pub fn unavailable_attachment_record(
    message_key: &str,
    ordinal: usize,
    filename_original: Option<String>,
    status: &str,
) -> AttachmentRecord {
    let attachment_key = ids::attachment_key(message_key, ordinal);
    let filename_safe = safe_filename(filename_original.as_deref(), ordinal);
    AttachmentRecord {
        message_key: message_key.to_string(),
        attachment_key: attachment_key.clone(),
        filename_original,
        filename_safe: filename_safe.clone(),
        content_type: None,
        extension: file_extension(&filename_safe),
        size_bytes: 0,
        sha256: sha256_hex(&[]),
        is_inline: false,
        content_id: None,
        ordinal: ordinal as u64,
        archive_path: format!("attachments/{message_key}/{attachment_key}_{filename_safe}"),
        extraction_status: status.to_string(),
    }
}

pub fn safe_filename(filename: Option<&str>, ordinal: usize) -> String {
    let fallback = format!("attachment_{ordinal}");
    let candidate = filename.unwrap_or(&fallback).trim();
    let mut safe = String::new();

    for ch in candidate.chars() {
        if ch.is_control() || matches!(ch, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
            safe.push('_');
        } else {
            safe.push(ch);
        }
    }

    let safe = safe
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_")
        .trim_matches('.')
        .trim_matches('_')
        .to_string();

    if safe.is_empty() {
        fallback
    } else {
        safe
    }
}

pub fn file_extension(filename: &str) -> Option<String> {
    filename
        .rsplit_once('.')
        .and_then(|(_, extension)| (!extension.is_empty()).then(|| extension.to_ascii_lowercase()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::{
        attachment_payload, file_extension, safe_filename, unavailable_attachment_record,
        AttachmentMetadata,
    };

    #[test]
    fn sanitizes_filenames() {
        assert_eq!(
            safe_filename(Some("report:final?.pdf"), 0),
            "report_final_.pdf"
        );
        assert_eq!(safe_filename(Some("../secret.txt"), 1), "secret.txt");
        assert_eq!(safe_filename(Some("   "), 2), "attachment_2");
        assert_eq!(safe_filename(None, 3), "attachment_3");
    }

    #[test]
    fn extracts_extensions() {
        assert_eq!(file_extension("file.PDF"), Some("pdf".to_string()));
        assert_eq!(file_extension("file"), None);
    }

    #[test]
    fn builds_attachment_payload() {
        let payload = attachment_payload(
            "msg_123",
            0,
            AttachmentMetadata {
                filename_original: Some("report.pdf".to_string()),
                content_type: Some("application/pdf".to_string()),
                is_inline: false,
                content_id: None,
            },
            b"pdf bytes".to_vec(),
        );

        assert_eq!(payload.record.message_key, "msg_123");
        assert_eq!(payload.record.filename_safe, "report.pdf");
        assert_eq!(payload.record.extension.as_deref(), Some("pdf"));
        assert_eq!(
            payload.record.content_type.as_deref(),
            Some("application/pdf")
        );
        assert_eq!(payload.record.size_bytes, 9);
        assert_eq!(payload.record.extraction_status, "extracted");
        assert_eq!(payload.bytes, b"pdf bytes");
    }

    #[test]
    fn builds_unavailable_attachment_record() {
        let record = unavailable_attachment_record(
            "msg_123",
            1,
            Some("missing.docx".to_string()),
            "payload_not_available",
        );

        assert_eq!(record.message_key, "msg_123");
        assert_eq!(record.filename_safe, "missing.docx");
        assert_eq!(record.extension.as_deref(), Some("docx"));
        assert_eq!(record.size_bytes, 0);
        assert_eq!(record.extraction_status, "payload_not_available");
    }

    #[test]
    fn preserves_inline_metadata() {
        let payload = attachment_payload(
            "msg_123",
            2,
            AttachmentMetadata {
                filename_original: Some("image.png".to_string()),
                content_type: Some("image/png".to_string()),
                is_inline: true,
                content_id: Some("cid-1".to_string()),
            },
            vec![1, 2, 3],
        );

        assert!(payload.record.is_inline);
        assert_eq!(payload.record.content_id.as_deref(), Some("cid-1"));
    }
}
