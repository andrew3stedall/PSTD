use sha2::{Digest, Sha256};

use crate::output::ids;
use crate::output::metadata::HeaderProjectionRecord;
use crate::pst::mapi::{MapiValue, PR_TRANSPORT_MESSAGE_HEADERS, PR_TRANSPORT_MESSAGE_HEADERS_A};
use crate::pst::property_context::PropertyContext;

const MAX_RAW_HEADER_BYTES: usize = 64 * 1024;

pub fn project(message_key: &str, properties: &PropertyContext) -> HeaderProjectionRecord {
    let (tag, source, charset_policy) = if properties.value(PR_TRANSPORT_MESSAGE_HEADERS).is_some()
    {
        (
            PR_TRANSPORT_MESSAGE_HEADERS,
            "stored_unicode",
            "unicode_utf16; readpst_item_charset_precedence",
        )
    } else if properties.value(PR_TRANSPORT_MESSAGE_HEADERS_A).is_some() {
        (
            PR_TRANSPORT_MESSAGE_HEADERS_A,
            "stored_string8",
            "string8_utf8_lossy; readpst_default_charset=iso-8859-1",
        )
    } else {
        return absent(message_key);
    };

    let property = properties
        .value(tag)
        .expect("header property selected above");
    let raw_evidence_key = Some(ids::stable_id(
        "evidence",
        &[message_key, "property", &format!("{tag:08x}")],
    ));
    let (raw_header_size_bytes, raw_header_sha256, raw_header_bytes_hex) =
        raw_fields(&property.raw);
    let header_key = ids::stable_id("header", &[message_key, source, &format!("{tag:08x}")]);

    let Some(decoded) = property.decoded.as_ref() else {
        return HeaderProjectionRecord {
            message_key: message_key.to_string(),
            header_key,
            source: source.to_string(),
            charset_policy: charset_policy.to_string(),
            raw_evidence_key,
            raw_header_size_bytes,
            raw_header_sha256,
            raw_header_bytes_hex,
            stored_headers: None,
            normalized_headers: None,
            validation_status: "stored_header_decode_failed".to_string(),
            authoritative: false,
            status: format!(
                "stored_header_decode_failed; raw_property_preserved; charset_policy={charset_policy}"
            ),
        };
    };

    let MapiValue::String(stored_headers) = decoded else {
        return invalid(
            message_key,
            header_key,
            source,
            charset_policy,
            raw_evidence_key,
            raw_header_size_bytes,
            raw_header_sha256,
            raw_header_bytes_hex,
            None,
            "stored_header_value_type_unsupported",
        );
    };

    let string8_encoding_status = if tag == PR_TRANSPORT_MESSAGE_HEADERS_A
        && std::str::from_utf8(trim_nul(&property.raw)).is_err()
    {
        Some("stored_string8_invalid_utf8_lossy")
    } else if tag == PR_TRANSPORT_MESSAGE_HEADERS && !unicode_raw_is_well_formed(&property.raw) {
        Some("stored_unicode_invalid_utf16_lossy")
    } else {
        None
    };

    let validation = validate_and_normalize(stored_headers);
    match validation {
        Ok((normalized_headers, embedded_body)) => {
            let validation_status = match (string8_encoding_status, embedded_body) {
                (Some(encoding), true) => format!("{encoding}; stored_valid_with_embedded_body"),
                (Some(encoding), false) => format!("{encoding}; stored_valid"),
                (None, true) => "stored_valid_with_embedded_body".to_string(),
                (None, false) => "stored_valid".to_string(),
            };
            HeaderProjectionRecord {
                message_key: message_key.to_string(),
                header_key,
                source: source.to_string(),
                charset_policy: charset_policy.to_string(),
                raw_evidence_key,
                raw_header_size_bytes,
                raw_header_sha256,
                raw_header_bytes_hex,
                stored_headers: Some(stored_headers.clone()),
                normalized_headers: Some(normalized_headers),
                validation_status: validation_status.clone(),
                authoritative: true,
                status: format!("{validation_status}; charset_policy={charset_policy}"),
            }
        }
        Err(reason) => invalid(
            message_key,
            header_key,
            source,
            charset_policy,
            raw_evidence_key,
            raw_header_size_bytes,
            raw_header_sha256,
            raw_header_bytes_hex,
            Some(stored_headers.clone()),
            &format!("stored_header_invalid_{reason}"),
        ),
    }
}

pub fn unavailable(message_key: &str, status: &str) -> HeaderProjectionRecord {
    let header_key = ids::stable_id("header", &[message_key, "unavailable"]);
    HeaderProjectionRecord {
        message_key: message_key.to_string(),
        header_key,
        source: "unavailable".to_string(),
        charset_policy: "readpst_default_charset=iso-8859-1".to_string(),
        raw_evidence_key: None,
        raw_header_size_bytes: 0,
        raw_header_sha256: None,
        raw_header_bytes_hex: None,
        stored_headers: None,
        normalized_headers: None,
        validation_status: status.to_string(),
        authoritative: false,
        status: format!("{status}; raw_header_unavailable"),
    }
}

fn absent(message_key: &str) -> HeaderProjectionRecord {
    let charset_policy = "readpst_default_charset=iso-8859-1";
    let header_key = ids::stable_id("header", &[message_key, "absent"]);
    HeaderProjectionRecord {
        message_key: message_key.to_string(),
        header_key,
        source: "absent".to_string(),
        charset_policy: charset_policy.to_string(),
        raw_evidence_key: None,
        raw_header_size_bytes: 0,
        raw_header_sha256: None,
        raw_header_bytes_hex: None,
        stored_headers: None,
        normalized_headers: None,
        validation_status: "stored_headers_absent".to_string(),
        authoritative: false,
        status: format!(
            "stored_headers_absent; typed_metadata_remains_authoritative; charset_policy={charset_policy}"
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn invalid(
    message_key: &str,
    header_key: String,
    source: &str,
    charset_policy: &str,
    raw_evidence_key: Option<String>,
    raw_header_size_bytes: u64,
    raw_header_sha256: Option<String>,
    raw_header_bytes_hex: Option<String>,
    stored_headers: Option<String>,
    validation_status: &str,
) -> HeaderProjectionRecord {
    HeaderProjectionRecord {
        message_key: message_key.to_string(),
        header_key,
        source: source.to_string(),
        charset_policy: charset_policy.to_string(),
        raw_evidence_key,
        raw_header_size_bytes,
        raw_header_sha256,
        raw_header_bytes_hex,
        stored_headers,
        normalized_headers: None,
        validation_status: validation_status.to_string(),
        authoritative: false,
        status: format!(
            "{validation_status}; stored_header_not_authoritative; raw_property_preserved; charset_policy={charset_policy}"
        ),
    }
}

fn validate_and_normalize(input: &str) -> Result<(String, bool), &'static str> {
    if input.is_empty() {
        return Err("empty");
    }
    if input.contains('\n') && input.replace("\r\n", "").contains('\n') {
        return Err("bare_line_feed");
    }
    if input.contains('\r') && !input.contains("\r\n") {
        return Err("bare_carriage_return");
    }

    let mut normalized = Vec::new();
    let mut previous_was_field = false;
    let mut saw_header = false;
    let mut embedded_body = false;
    for line in input.split("\r\n") {
        if line.is_empty() {
            if saw_header {
                embedded_body = input
                    .split_once("\r\n\r\n")
                    .map(|(_, body)| !body.is_empty())
                    .unwrap_or(false);
                break;
            }
            return Err("leading_blank_line");
        }
        if line.starts_with(' ') || line.starts_with('\t') {
            if !previous_was_field {
                return Err("orphan_continuation");
            }
            if line
                .chars()
                .any(|character| character.is_control() && character != '\t')
            {
                return Err("control_character");
            }
            normalized.push(line.to_string());
            continue;
        }

        let Some(colon) = line.find(':') else {
            return Err("missing_colon");
        };
        let name = &line[..colon];
        if name.is_empty()
            || !name
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '-')
        {
            return Err("invalid_field_name");
        }
        if line[colon + 1..]
            .chars()
            .any(|character| character.is_control() && character != '\t')
        {
            return Err("control_character");
        }
        normalized.push(line.to_string());
        previous_was_field = true;
        saw_header = true;
    }

    if !saw_header || normalized.is_empty() {
        return Err("no_fields");
    }
    Ok((normalized.join("\n"), embedded_body))
}

fn trim_nul(raw: &[u8]) -> &[u8] {
    let end = raw.iter().position(|byte| *byte == 0).unwrap_or(raw.len());
    &raw[..end]
}

#[allow(clippy::chunks_exact_to_as_chunks)]
fn unicode_raw_is_well_formed(raw: &[u8]) -> bool {
    if !raw.len().is_multiple_of(2) {
        return false;
    }
    let units = raw
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .take_while(|unit| *unit != 0)
        .collect::<Vec<_>>();
    String::from_utf16(&units).is_ok()
}

fn raw_fields(raw: &[u8]) -> (u64, Option<String>, Option<String>) {
    let mut hasher = Sha256::new();
    hasher.update(raw);
    let digest = hex::encode(hasher.finalize());
    let bounded = &raw[..raw.len().min(MAX_RAW_HEADER_BYTES)];
    (raw.len() as u64, Some(digest), Some(hex::encode(bounded)))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::project;
    use crate::pst::mapi::{
        MapiValue, PR_TRANSPORT_MESSAGE_HEADERS, PR_TRANSPORT_MESSAGE_HEADERS_A,
    };
    use crate::pst::property_context::{PropertyContext, PropertyValue};

    fn context(tag: u32, raw: Vec<u8>, value: Option<&str>) -> PropertyContext {
        let mut values = HashMap::new();
        values.insert(
            tag,
            PropertyValue {
                tag,
                name: "transport_message_headers".to_string(),
                raw,
                decoded: value.map(|value| MapiValue::String(value.to_string())),
                status: "selected".to_string(),
            },
        );
        PropertyContext::from_values(values)
    }

    #[test]
    fn preserves_valid_folded_headers_and_normalizes_line_endings() {
        let record = project(
            "msg-valid",
            &context(
                PR_TRANSPORT_MESSAGE_HEADERS,
                b"From: sender@example.com\r\nSubject: hello\r\n\tcontinued\r\n".to_vec(),
                Some("From: sender@example.com\r\nSubject: hello\r\n\tcontinued\r\n"),
            ),
        );
        assert!(record.authoritative);
        assert_eq!(record.validation_status, "stored_valid");
        assert_eq!(
            record.normalized_headers.as_deref(),
            Some("From: sender@example.com\nSubject: hello\n\tcontinued")
        );
        assert_eq!(record.raw_header_size_bytes, 54);
        assert!(record.raw_evidence_key.is_some());
    }

    #[test]
    fn rejects_body_fragments_and_preserves_the_stored_value() {
        let value = "body fragment\r\n";
        let record = project(
            "msg-invalid",
            &context(
                PR_TRANSPORT_MESSAGE_HEADERS,
                value.as_bytes().to_vec(),
                Some(value),
            ),
        );
        assert!(!record.authoritative);
        assert!(record.validation_status.contains("missing_colon"));
        assert_eq!(record.stored_headers.as_deref(), Some(value));
        assert!(record.raw_header_bytes_hex.is_some());
    }

    #[test]
    fn records_string8_policy_and_decode_failures_without_dropping_raw_bytes() {
        let invalid_utf8 = context(
            PR_TRANSPORT_MESSAGE_HEADERS_A,
            vec![0xff, 0x00],
            Some("From: replacement@example.com"),
        );
        let record = project("msg-string8", &invalid_utf8);
        assert!(record.authoritative);
        assert!(record.status.contains("stored_string8_invalid_utf8_lossy"));
        assert!(record.charset_policy.contains("iso-8859-1"));

        let failed = project(
            "msg-decode-failed",
            &context(PR_TRANSPORT_MESSAGE_HEADERS, vec![0x41], None),
        );
        assert!(!failed.authoritative);
        assert_eq!(failed.validation_status, "stored_header_decode_failed");
        assert_eq!(failed.raw_header_bytes_hex.as_deref(), Some("41"));
    }
}
