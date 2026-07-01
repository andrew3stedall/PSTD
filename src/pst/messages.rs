use sha2::{Digest, Sha256};

use crate::output::ids;
use crate::output::metadata::BodyRecord;

#[derive(Debug, Clone)]
pub struct BodyPayload {
    pub record: BodyRecord,
    pub bytes: Vec<u8>,
}

pub fn text_body_payload(message_key: &str, text: &str) -> BodyPayload {
    body_payload(message_key, "text", text.as_bytes().to_vec(), Some("utf-8"))
}

pub fn html_body_payload(message_key: &str, html: &[u8]) -> BodyPayload {
    body_payload(message_key, "html", html.to_vec(), None)
}

pub fn body_payload(
    message_key: &str,
    body_type: &str,
    bytes: Vec<u8>,
    encoding: Option<&str>,
) -> BodyPayload {
    let body_key = ids::body_key(message_key, body_type);
    let extension = body_extension(body_type);
    let archive_path = format!("bodies/{message_key}.{extension}");
    let sha256 = sha256_hex(&bytes);

    BodyPayload {
        record: BodyRecord {
            message_key: message_key.to_string(),
            body_key,
            body_type: body_type.to_string(),
            archive_path,
            encoding: encoding.map(ToString::to_string),
            size_bytes: bytes.len() as u64,
            sha256,
            status: "extracted".to_string(),
        },
        bytes,
    }
}

pub fn unavailable_body_record(message_key: &str, body_type: &str, status: &str) -> BodyRecord {
    BodyRecord {
        message_key: message_key.to_string(),
        body_key: ids::body_key(message_key, body_type),
        body_type: body_type.to_string(),
        archive_path: format!("bodies/{message_key}.{}", body_extension(body_type)),
        encoding: None,
        size_bytes: 0,
        sha256: sha256_hex(&[]),
        status: status.to_string(),
    }
}

pub fn body_extension(body_type: &str) -> &'static str {
    match body_type {
        "html" => "html",
        "rtf" => "rtf",
        _ => "txt",
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::{body_extension, html_body_payload, text_body_payload, unavailable_body_record};

    #[test]
    fn builds_text_body_payload() {
        let payload = text_body_payload("msg_123", "Hello world");
        assert_eq!(payload.record.message_key, "msg_123");
        assert_eq!(payload.record.body_type, "text");
        assert_eq!(payload.record.archive_path, "bodies/msg_123.txt");
        assert_eq!(payload.record.encoding.as_deref(), Some("utf-8"));
        assert_eq!(payload.record.size_bytes, 11);
        assert_eq!(payload.record.status, "extracted");
        assert_eq!(payload.bytes, b"Hello world");
    }

    #[test]
    fn builds_html_body_payload() {
        let payload = html_body_payload("msg_123", b"<p>Hello</p>");
        assert_eq!(payload.record.body_type, "html");
        assert_eq!(payload.record.archive_path, "bodies/msg_123.html");
        assert_eq!(payload.record.encoding, None);
        assert_eq!(payload.record.size_bytes, 12);
    }

    #[test]
    fn builds_unavailable_body_record() {
        let record = unavailable_body_record("msg_123", "text", "payload_not_available");
        assert_eq!(record.message_key, "msg_123");
        assert_eq!(record.body_type, "text");
        assert_eq!(record.archive_path, "bodies/msg_123.txt");
        assert_eq!(record.size_bytes, 0);
        assert_eq!(record.status, "payload_not_available");
    }

    #[test]
    fn maps_body_extensions() {
        assert_eq!(body_extension("text"), "txt");
        assert_eq!(body_extension("html"), "html");
        assert_eq!(body_extension("rtf"), "rtf");
        assert_eq!(body_extension("other"), "txt");
    }
}
