use sha2::{Digest, Sha256};

use crate::output::ids;
use crate::output::metadata::AttachmentRecord;
use crate::pst::mapi::{
    MapiValue, PR_ATTACHMENT_HIDDEN, PR_ATTACH_CONTENT_ID, PR_ATTACH_DATA_BIN, PR_ATTACH_FILENAME,
    PR_ATTACH_LONG_FILENAME, PR_ATTACH_METHOD, PR_ATTACH_MIME_TAG, PR_ATTACH_SIZE,
};
use crate::pst::property_context::PropertyContext;

pub const ATTACH_METHOD_EMBEDDED_MESSAGE: i32 = 5;

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
    pub attachment_method: Option<i32>,
    pub declared_size_bytes: Option<u64>,
}

pub fn attachment_payload_from_properties(
    message_key: &str,
    ordinal: usize,
    properties: &PropertyContext,
) -> Option<AttachmentPayload> {
    let bytes = binary_property_bytes(properties, PR_ATTACH_DATA_BIN)?;
    let metadata = attachment_metadata_from_properties(properties);

    Some(attachment_payload(message_key, ordinal, metadata, bytes))
}

pub fn unavailable_attachment_record_from_properties(
    message_key: &str,
    ordinal: usize,
    properties: &PropertyContext,
    status: &str,
) -> AttachmentRecord {
    let metadata = attachment_metadata_from_properties(properties);
    unavailable_attachment_record_from_metadata(message_key, ordinal, metadata, status)
}

pub fn attachment_metadata_from_properties(properties: &PropertyContext) -> AttachmentMetadata {
    let content_id = properties.string_value(PR_ATTACH_CONTENT_ID);
    let is_hidden = bool_property(properties, PR_ATTACHMENT_HIDDEN).unwrap_or(false);
    AttachmentMetadata {
        filename_original: properties
            .string_value(PR_ATTACH_LONG_FILENAME)
            .or_else(|| properties.string_value(PR_ATTACH_FILENAME)),
        content_type: properties.string_value(PR_ATTACH_MIME_TAG),
        is_inline: is_hidden || content_id.is_some(),
        content_id,
        attachment_method: i32_property(properties, PR_ATTACH_METHOD),
        declared_size_bytes: i32_property(properties, PR_ATTACH_SIZE)
            .and_then(|value| u64::try_from(value).ok()),
    }
}

pub fn attachment_payload(
    message_key: &str,
    ordinal: usize,
    metadata: AttachmentMetadata,
    bytes: Vec<u8>,
) -> AttachmentPayload {
    let record = attachment_record(message_key, ordinal, metadata, Some(&bytes), "extracted");

    AttachmentPayload { record, bytes }
}

pub fn unavailable_attachment_record_from_metadata(
    message_key: &str,
    ordinal: usize,
    metadata: AttachmentMetadata,
    status: &str,
) -> AttachmentRecord {
    let status = if metadata.attachment_method == Some(ATTACH_METHOD_EMBEDDED_MESSAGE) {
        "embedded_message_payload_deferred"
    } else {
        status
    };
    attachment_record(message_key, ordinal, metadata, None, status)
}

pub fn unavailable_attachment_record(
    message_key: &str,
    ordinal: usize,
    filename_original: Option<String>,
    status: &str,
) -> AttachmentRecord {
    unavailable_attachment_record_from_metadata(
        message_key,
        ordinal,
        AttachmentMetadata {
            filename_original,
            ..AttachmentMetadata::default()
        },
        status,
    )
}

fn attachment_record(
    message_key: &str,
    ordinal: usize,
    metadata: AttachmentMetadata,
    bytes: Option<&[u8]>,
    status: &str,
) -> AttachmentRecord {
    let attachment_key = ids::attachment_key(message_key, ordinal);
    let filename_safe = safe_filename(metadata.filename_original.as_deref(), ordinal);
    let extension = file_extension(&filename_safe);
    let archive_path = format!("attachments/{message_key}/{attachment_key}_{filename_safe}");
    let payload_bytes = bytes.unwrap_or(&[]);
    let sha256 = sha256_hex(payload_bytes);
    let size_bytes = payload_bytes.len() as u64;
    let size_status = size_status(
        metadata.declared_size_bytes,
        bytes.map(|value| value.len() as u64),
    );

    AttachmentRecord {
        message_key: message_key.to_string(),
        attachment_key,
        filename_original: metadata.filename_original,
        filename_safe,
        content_type: metadata.content_type,
        extension,
        size_bytes,
        declared_size_bytes: metadata.declared_size_bytes,
        size_status,
        sha256,
        is_inline: metadata.is_inline,
        content_id: metadata.content_id,
        attachment_method: metadata.attachment_method,
        ordinal: ordinal as u64,
        archive_path,
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

fn binary_property_bytes(properties: &PropertyContext, tag: u32) -> Option<Vec<u8>> {
    let value = properties.value(tag)?;
    match value.decoded.as_ref() {
        Some(MapiValue::Binary(bytes)) => Some(bytes.clone()),
        _ if !value.raw.is_empty() => Some(value.raw.clone()),
        _ => None,
    }
}

fn bool_property(properties: &PropertyContext, tag: u32) -> Option<bool> {
    let value = properties.value(tag)?;
    match value.decoded.as_ref() {
        Some(MapiValue::Boolean(value)) => Some(*value),
        _ => None,
    }
}

fn i32_property(properties: &PropertyContext, tag: u32) -> Option<i32> {
    let value = properties.value(tag)?;
    match value.decoded.as_ref() {
        Some(MapiValue::Integer32(value)) => Some(*value),
        _ => None,
    }
}

fn size_status(declared_size_bytes: Option<u64>, actual_size_bytes: Option<u64>) -> String {
    match (declared_size_bytes, actual_size_bytes) {
        (Some(declared), Some(actual)) if declared == actual => "size_matched".to_string(),
        (Some(_), Some(_)) => "size_mismatch".to_string(),
        (Some(_), None) => "payload_unavailable_declared_size_present".to_string(),
        (None, Some(_)) => "declared_size_absent".to_string(),
        (None, None) => "payload_unavailable_size_unknown".to_string(),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{
        attachment_payload, attachment_payload_from_properties, file_extension, safe_filename,
        unavailable_attachment_record, unavailable_attachment_record_from_properties,
        AttachmentMetadata, ATTACH_METHOD_EMBEDDED_MESSAGE,
    };
    use crate::pst::mapi::{
        MapiValue, PR_ATTACH_CONTENT_ID, PR_ATTACH_DATA_BIN, PR_ATTACH_LONG_FILENAME,
        PR_ATTACH_METHOD, PR_ATTACH_MIME_TAG, PR_ATTACH_SIZE,
    };
    use crate::pst::property_context::{PropertyContext, PropertyValue};

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
                attachment_method: Some(1),
                declared_size_bytes: Some(9),
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
        assert_eq!(payload.record.declared_size_bytes, Some(9));
        assert_eq!(payload.record.size_status, "size_matched");
        assert_eq!(payload.record.attachment_method, Some(1));
        assert_eq!(payload.record.extraction_status, "extracted");
        assert_eq!(payload.bytes, b"pdf bytes");
    }

    #[test]
    fn builds_attachment_payload_from_properties() {
        let mut values = HashMap::new();
        values.insert(
            PR_ATTACH_DATA_BIN,
            PropertyValue {
                tag: PR_ATTACH_DATA_BIN,
                name: "attachment_data".to_string(),
                raw: b"image-bytes".to_vec(),
                decoded: Some(MapiValue::Binary(b"image-bytes".to_vec())),
                status: "selected".to_string(),
            },
        );
        values.insert(
            PR_ATTACH_LONG_FILENAME,
            PropertyValue {
                tag: PR_ATTACH_LONG_FILENAME,
                name: "attachment_long_filename".to_string(),
                raw: Vec::new(),
                decoded: Some(MapiValue::String("image.png".to_string())),
                status: "selected".to_string(),
            },
        );
        values.insert(
            PR_ATTACH_MIME_TAG,
            PropertyValue {
                tag: PR_ATTACH_MIME_TAG,
                name: "attachment_mime_tag".to_string(),
                raw: Vec::new(),
                decoded: Some(MapiValue::String("image/png".to_string())),
                status: "selected".to_string(),
            },
        );
        values.insert(
            PR_ATTACH_CONTENT_ID,
            PropertyValue {
                tag: PR_ATTACH_CONTENT_ID,
                name: "attachment_content_id".to_string(),
                raw: Vec::new(),
                decoded: Some(MapiValue::String("cid-1".to_string())),
                status: "selected".to_string(),
            },
        );
        values.insert(
            PR_ATTACH_SIZE,
            PropertyValue {
                tag: PR_ATTACH_SIZE,
                name: "attachment_size".to_string(),
                raw: 11i32.to_le_bytes().to_vec(),
                decoded: Some(MapiValue::Integer32(11)),
                status: "selected".to_string(),
            },
        );
        values.insert(
            PR_ATTACH_METHOD,
            PropertyValue {
                tag: PR_ATTACH_METHOD,
                name: "attachment_method".to_string(),
                raw: 1i32.to_le_bytes().to_vec(),
                decoded: Some(MapiValue::Integer32(1)),
                status: "selected".to_string(),
            },
        );
        let properties = PropertyContext { values };

        let payload = attachment_payload_from_properties("msg_123", 0, &properties).unwrap();
        assert_eq!(payload.record.filename_safe, "image.png");
        assert_eq!(payload.record.content_type.as_deref(), Some("image/png"));
        assert!(payload.record.is_inline);
        assert_eq!(payload.record.content_id.as_deref(), Some("cid-1"));
        assert_eq!(payload.record.declared_size_bytes, Some(11));
        assert_eq!(payload.record.size_status, "size_matched");
        assert_eq!(payload.record.attachment_method, Some(1));
        assert_eq!(payload.bytes, b"image-bytes");
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
        assert_eq!(record.declared_size_bytes, None);
        assert_eq!(record.size_status, "payload_unavailable_size_unknown");
        assert_eq!(record.extraction_status, "payload_not_available");
    }

    #[test]
    fn builds_unavailable_attachment_record_from_properties() {
        let mut values = HashMap::new();
        values.insert(
            PR_ATTACH_LONG_FILENAME,
            PropertyValue {
                tag: PR_ATTACH_LONG_FILENAME,
                name: "attachment_long_filename".to_string(),
                raw: Vec::new(),
                decoded: Some(MapiValue::String("missing.pdf".to_string())),
                status: "selected".to_string(),
            },
        );
        values.insert(
            PR_ATTACH_SIZE,
            PropertyValue {
                tag: PR_ATTACH_SIZE,
                name: "attachment_size".to_string(),
                raw: 42i32.to_le_bytes().to_vec(),
                decoded: Some(MapiValue::Integer32(42)),
                status: "selected".to_string(),
            },
        );
        values.insert(
            PR_ATTACH_METHOD,
            PropertyValue {
                tag: PR_ATTACH_METHOD,
                name: "attachment_method".to_string(),
                raw: ATTACH_METHOD_EMBEDDED_MESSAGE.to_le_bytes().to_vec(),
                decoded: Some(MapiValue::Integer32(ATTACH_METHOD_EMBEDDED_MESSAGE)),
                status: "selected".to_string(),
            },
        );
        let properties = PropertyContext { values };

        let record = unavailable_attachment_record_from_properties(
            "msg_123",
            1,
            &properties,
            "attachment_payload_property_absent",
        );

        assert_eq!(record.filename_safe, "missing.pdf");
        assert_eq!(record.declared_size_bytes, Some(42));
        assert_eq!(
            record.size_status,
            "payload_unavailable_declared_size_present"
        );
        assert_eq!(
            record.attachment_method,
            Some(ATTACH_METHOD_EMBEDDED_MESSAGE)
        );
        assert_eq!(
            record.extraction_status,
            "embedded_message_payload_deferred"
        );
    }

    #[test]
    fn marks_size_mismatches() {
        let payload = attachment_payload(
            "msg_123",
            3,
            AttachmentMetadata {
                filename_original: Some("bad-size.bin".to_string()),
                content_type: None,
                is_inline: false,
                content_id: None,
                attachment_method: Some(1),
                declared_size_bytes: Some(100),
            },
            vec![1, 2, 3],
        );

        assert_eq!(payload.record.size_bytes, 3);
        assert_eq!(payload.record.declared_size_bytes, Some(100));
        assert_eq!(payload.record.size_status, "size_mismatch");
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
                attachment_method: None,
                declared_size_bytes: None,
            },
            vec![1, 2, 3],
        );

        assert!(payload.record.is_inline);
        assert_eq!(payload.record.content_id.as_deref(), Some("cid-1"));
    }
}
