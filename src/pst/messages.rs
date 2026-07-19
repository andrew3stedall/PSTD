use sha2::{Digest, Sha256};

use crate::output::ids;
use crate::output::metadata::BodyRecord;
use crate::pst::mapi::{
    MapiValue, PR_BODY, PR_BODY_A, PR_HTML, PR_HTML_STRING, PR_HTML_STRING_A, PR_RTF_COMPRESSED,
};
use crate::pst::property_context::PropertyContext;

const MAX_BINARY_BODY_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct BodyPayload {
    pub record: BodyRecord,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BodyCoverageReport {
    pub text_property_present: bool,
    pub html_property_present: bool,
    pub rtf_property_present: bool,
    pub supported_body_property_count: usize,
    pub extracted_payload_count: usize,
    pub fallback_record_count: usize,
    pub unresolved_body_types: Vec<String>,
    pub preferred_body_type: Option<String>,
    pub status: String,
}

pub fn body_payloads_from_properties(
    message_key: &str,
    properties: &PropertyContext,
) -> Vec<BodyPayload> {
    let mut payloads = Vec::new();

    if let Some(text) = properties.first_string_value(&[PR_BODY, PR_BODY_A]) {
        payloads.push(text_body_payload(message_key, &text));
    }

    if let Some(html) = binary_property_bytes(properties, PR_HTML) {
        payloads.push(html_body_payload(message_key, &html));
    } else if let Some(html) = properties.first_string_value(&[PR_HTML_STRING, PR_HTML_STRING_A]) {
        payloads.push(html_string_body_payload(message_key, &html));
    }

    if let Some(rtf) = binary_property_bytes(properties, PR_RTF_COMPRESSED) {
        payloads.push(body_payload(message_key, "rtf", rtf, None));
    }

    payloads
}

pub fn body_coverage_report(
    properties: &PropertyContext,
    payloads: &[BodyPayload],
) -> BodyCoverageReport {
    let text_property_present =
        properties.value(PR_BODY).is_some() || properties.value(PR_BODY_A).is_some();
    let html_property_present = properties.value(PR_HTML).is_some()
        || properties.value(PR_HTML_STRING).is_some()
        || properties.value(PR_HTML_STRING_A).is_some();
    let rtf_property_present = properties.value(PR_RTF_COMPRESSED).is_some();
    let supported_body_property_count = [
        text_property_present,
        html_property_present,
        rtf_property_present,
    ]
    .iter()
    .filter(|present| **present)
    .count();
    let extracted_payload_count = payloads.len();
    let unresolved_body_types = unresolved_body_types(
        text_property_present,
        html_property_present,
        rtf_property_present,
        payloads,
    );
    let preferred_body_type = preferred_body_type(payloads).map(ToString::to_string);
    let fallback_record_count = if supported_body_property_count == 0 {
        1
    } else {
        unresolved_body_types.len()
    };
    let status = if extracted_payload_count > 0 && !unresolved_body_types.is_empty() {
        format!(
            "body_payload_extracted; payloads={extracted_payload_count}; supported_body_properties={supported_body_property_count}; body_types={}; preferred_body_type={}; unresolved_body_types={}",
            body_type_summary(payloads),
            preferred_body_type.as_deref().unwrap_or("none"),
            unresolved_body_types.join(",")
        )
    } else if extracted_payload_count > 0 {
        format!(
            "body_payload_extracted; payloads={extracted_payload_count}; supported_body_properties={supported_body_property_count}; body_types={}",
            body_type_summary(payloads)
        )
    } else if supported_body_property_count > 0 {
        format!(
            "body_payload_properties_present_but_unusable; supported_body_properties={supported_body_property_count}"
        )
    } else {
        "body_payload_property_absent; supported_body_properties=0".to_string()
    };

    BodyCoverageReport {
        text_property_present,
        html_property_present,
        rtf_property_present,
        supported_body_property_count,
        extracted_payload_count,
        fallback_record_count,
        unresolved_body_types,
        preferred_body_type,
        status,
    }
}

pub fn unresolved_body_records(message_key: &str, report: &BodyCoverageReport) -> Vec<BodyRecord> {
    report
        .unresolved_body_types
        .iter()
        .map(|body_type| {
            unavailable_body_record(
                message_key,
                body_type,
                &format!("body_payload_property_present_but_unresolved; body_type={body_type}"),
            )
        })
        .collect()
}

pub fn text_body_payload(message_key: &str, text: &str) -> BodyPayload {
    body_payload(message_key, "text", text.as_bytes().to_vec(), Some("utf-8"))
}

pub fn html_body_payload(message_key: &str, html: &[u8]) -> BodyPayload {
    body_payload(message_key, "html", html.to_vec(), None)
}

pub fn html_string_body_payload(message_key: &str, html: &str) -> BodyPayload {
    body_payload(message_key, "html", html.as_bytes().to_vec(), Some("utf-8"))
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

fn unresolved_body_types(
    text_property_present: bool,
    html_property_present: bool,
    rtf_property_present: bool,
    payloads: &[BodyPayload],
) -> Vec<String> {
    [
        ("text", text_property_present),
        ("html", html_property_present),
        ("rtf", rtf_property_present),
    ]
    .into_iter()
    .filter(|(body_type, present)| {
        *present
            && !payloads
                .iter()
                .any(|payload| payload.record.body_type == *body_type)
    })
    .map(|(body_type, _)| body_type.to_string())
    .collect()
}

fn preferred_body_type(payloads: &[BodyPayload]) -> Option<&'static str> {
    ["html", "text", "rtf"].into_iter().find(|body_type| {
        payloads
            .iter()
            .any(|payload| payload.record.body_type == *body_type)
    })
}

fn body_type_summary(payloads: &[BodyPayload]) -> String {
    let mut body_types = payloads
        .iter()
        .map(|payload| payload.record.body_type.as_str())
        .collect::<Vec<_>>();
    body_types.sort_unstable();
    body_types.dedup();
    if body_types.is_empty() {
        "none".to_string()
    } else {
        body_types.join(",")
    }
}

fn binary_property_bytes(properties: &PropertyContext, tag: u32) -> Option<Vec<u8>> {
    let value = properties.value(tag)?;
    match value.decoded.as_ref() {
        Some(MapiValue::Binary(bytes))
            if !bytes.is_empty() && bytes.len() <= MAX_BINARY_BODY_BYTES =>
        {
            Some(bytes.clone())
        }
        _ => None,
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
        body_coverage_report, body_extension, body_payloads_from_properties, html_body_payload,
        html_string_body_payload, text_body_payload, unavailable_body_record,
        unresolved_body_records,
    };
    use crate::pst::mapi::{
        MapiValue, PR_BODY, PR_BODY_A, PR_HTML, PR_HTML_STRING, PR_HTML_STRING_A, PR_RTF_COMPRESSED,
    };
    use crate::pst::property_context::{PropertyContext, PropertyValue};

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
    fn builds_unicode_html_body_payload() {
        let payload = html_string_body_payload("msg_123", "<p>Hello</p>");
        assert_eq!(payload.record.body_type, "html");
        assert_eq!(payload.record.archive_path, "bodies/msg_123.html");
        assert_eq!(payload.record.encoding.as_deref(), Some("utf-8"));
        assert_eq!(payload.record.size_bytes, 12);
        assert_eq!(payload.bytes, b"<p>Hello</p>");
    }

    #[test]
    fn builds_body_payloads_from_properties() {
        let mut values = HashMap::new();
        values.insert(
            PR_BODY,
            PropertyValue {
                tag: PR_BODY,
                name: "body_text".to_string(),
                raw: Vec::new(),
                decoded: Some(MapiValue::String("Hello".to_string())),
                status: "selected".to_string(),
            },
        );
        values.insert(
            PR_HTML,
            PropertyValue {
                tag: PR_HTML,
                name: "body_html".to_string(),
                raw: b"<p>Hello</p>".to_vec(),
                decoded: Some(MapiValue::Binary(b"<p>Hello</p>".to_vec())),
                status: "selected".to_string(),
            },
        );
        let properties = PropertyContext::from_values(values);

        let payloads = body_payloads_from_properties("msg_123", &properties);
        assert_eq!(payloads.len(), 2);
        assert_eq!(payloads[0].record.body_type, "text");
        assert_eq!(payloads[1].record.body_type, "html");
    }

    #[test]
    fn builds_body_payloads_from_string8_alias_properties() {
        let mut values = HashMap::new();
        values.insert(
            PR_BODY_A,
            PropertyValue {
                tag: PR_BODY_A,
                name: "body_text".to_string(),
                raw: Vec::new(),
                decoded: Some(MapiValue::String("Hello alias".to_string())),
                status: "selected".to_string(),
            },
        );
        values.insert(
            PR_HTML_STRING_A,
            PropertyValue {
                tag: PR_HTML_STRING_A,
                name: "body_html_unicode".to_string(),
                raw: Vec::new(),
                decoded: Some(MapiValue::String("<p>Alias</p>".to_string())),
                status: "selected".to_string(),
            },
        );
        let properties = PropertyContext::from_values(values);

        let payloads = body_payloads_from_properties("msg_123", &properties);
        let report = body_coverage_report(&properties, &payloads);

        assert_eq!(payloads.len(), 2);
        assert_eq!(payloads[0].record.body_type, "text");
        assert_eq!(payloads[1].record.body_type, "html");
        assert!(report.text_property_present);
        assert!(report.html_property_present);
        assert_eq!(report.supported_body_property_count, 2);
    }

    #[test]
    fn leaves_undecoded_binary_reference_unresolved() {
        let mut values = HashMap::new();
        values.insert(
            PR_HTML,
            PropertyValue {
                tag: PR_HTML,
                name: "body_html".to_string(),
                raw: vec![0x7f, 0x80, 0x00, 0x00],
                decoded: None,
                status: "selected".to_string(),
            },
        );
        let properties = PropertyContext::from_values(values);

        let payloads = body_payloads_from_properties("msg_123", &properties);
        let report = body_coverage_report(&properties, &payloads);
        let records = unresolved_body_records("msg_123", &report);

        assert!(payloads.is_empty());
        assert_eq!(report.unresolved_body_types, vec!["html"]);
        assert_eq!(report.preferred_body_type, None);
        assert_eq!(report.fallback_record_count, 1);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].body_type, "html");
        assert_eq!(records[0].size_bytes, 0);
        assert_eq!(
            records[0].status,
            "body_payload_property_present_but_unresolved; body_type=html"
        );
    }

    #[test]
    fn selects_plain_text_when_html_is_unresolved() {
        let mut values = HashMap::new();
        values.insert(
            PR_BODY,
            PropertyValue {
                tag: PR_BODY,
                name: "body_text".to_string(),
                raw: Vec::new(),
                decoded: Some(MapiValue::String("Plain body".to_string())),
                status: "selected".to_string(),
            },
        );
        values.insert(
            PR_HTML,
            PropertyValue {
                tag: PR_HTML,
                name: "body_html".to_string(),
                raw: vec![0x7f, 0x80, 0x00, 0x00],
                decoded: None,
                status: "selected".to_string(),
            },
        );
        let properties = PropertyContext::from_values(values);

        let payloads = body_payloads_from_properties("msg_123", &properties);
        let report = body_coverage_report(&properties, &payloads);

        assert_eq!(payloads.len(), 1);
        assert_eq!(payloads[0].record.body_type, "text");
        assert_eq!(report.preferred_body_type.as_deref(), Some("text"));
        assert_eq!(report.unresolved_body_types, vec!["html"]);
        assert_eq!(report.fallback_record_count, 1);
        assert_eq!(
            report.status,
            "body_payload_extracted; payloads=1; supported_body_properties=2; body_types=text; preferred_body_type=text; unresolved_body_types=html"
        );
    }

    #[test]
    fn builds_html_body_payload_from_unicode_property() {
        let mut values = HashMap::new();
        values.insert(
            PR_HTML_STRING,
            PropertyValue {
                tag: PR_HTML_STRING,
                name: "body_html_unicode".to_string(),
                raw: Vec::new(),
                decoded: Some(MapiValue::String("<p>Hello Unicode</p>".to_string())),
                status: "selected".to_string(),
            },
        );
        let properties = PropertyContext::from_values(values);

        let payloads = body_payloads_from_properties("msg_123", &properties);
        assert_eq!(payloads.len(), 1);
        assert_eq!(payloads[0].record.body_type, "html");
        assert_eq!(payloads[0].record.encoding.as_deref(), Some("utf-8"));
        assert_eq!(payloads[0].bytes, b"<p>Hello Unicode</p>");
    }

    #[test]
    fn prefers_binary_html_over_unicode_html_when_both_are_present() {
        let mut values = HashMap::new();
        values.insert(
            PR_HTML,
            PropertyValue {
                tag: PR_HTML,
                name: "body_html".to_string(),
                raw: b"<p>Binary</p>".to_vec(),
                decoded: Some(MapiValue::Binary(b"<p>Binary</p>".to_vec())),
                status: "selected".to_string(),
            },
        );
        values.insert(
            PR_HTML_STRING,
            PropertyValue {
                tag: PR_HTML_STRING,
                name: "body_html_unicode".to_string(),
                raw: Vec::new(),
                decoded: Some(MapiValue::String("<p>Unicode</p>".to_string())),
                status: "selected".to_string(),
            },
        );
        let properties = PropertyContext::from_values(values);

        let payloads = body_payloads_from_properties("msg_123", &properties);
        assert_eq!(payloads.len(), 1);
        assert_eq!(payloads[0].record.encoding, None);
        assert_eq!(payloads[0].bytes, b"<p>Binary</p>");
    }

    #[test]
    fn builds_body_payloads_from_all_supported_synthetic_properties() {
        let mut values = HashMap::new();
        values.insert(
            PR_BODY,
            PropertyValue {
                tag: PR_BODY,
                name: "body_text".to_string(),
                raw: Vec::new(),
                decoded: Some(MapiValue::String("Hello".to_string())),
                status: "selected".to_string(),
            },
        );
        values.insert(
            PR_HTML,
            PropertyValue {
                tag: PR_HTML,
                name: "body_html".to_string(),
                raw: b"<p>Hello</p>".to_vec(),
                decoded: Some(MapiValue::Binary(b"<p>Hello</p>".to_vec())),
                status: "selected".to_string(),
            },
        );
        values.insert(
            PR_RTF_COMPRESSED,
            PropertyValue {
                tag: PR_RTF_COMPRESSED,
                name: "rtf_compressed".to_string(),
                raw: b"{\\rtf1 synthetic}".to_vec(),
                decoded: Some(MapiValue::Binary(b"{\\rtf1 synthetic}".to_vec())),
                status: "selected".to_string(),
            },
        );
        let properties = PropertyContext::from_values(values);

        let payloads = body_payloads_from_properties("msg_123", &properties);
        assert_eq!(payloads.len(), 3);
        assert_eq!(payloads[0].record.archive_path, "bodies/msg_123.txt");
        assert_eq!(payloads[1].record.archive_path, "bodies/msg_123.html");
        assert_eq!(payloads[2].record.archive_path, "bodies/msg_123.rtf");
        assert_eq!(payloads[2].bytes, b"{\\rtf1 synthetic}");
    }

    #[test]
    fn reports_extracted_body_coverage() {
        let mut values = HashMap::new();
        values.insert(
            PR_BODY,
            PropertyValue {
                tag: PR_BODY,
                name: "body_text".to_string(),
                raw: Vec::new(),
                decoded: Some(MapiValue::String("Hello".to_string())),
                status: "selected".to_string(),
            },
        );
        let properties = PropertyContext::from_values(values);
        let payloads = body_payloads_from_properties("msg_123", &properties);
        let report = body_coverage_report(&properties, &payloads);

        assert!(report.text_property_present);
        assert_eq!(report.supported_body_property_count, 1);
        assert_eq!(report.extracted_payload_count, 1);
        assert_eq!(report.fallback_record_count, 0);
        assert_eq!(
            report.status,
            "body_payload_extracted; payloads=1; supported_body_properties=1; body_types=text"
        );
    }

    #[test]
    fn reports_absent_body_coverage() {
        let properties = PropertyContext::default();
        let payloads = body_payloads_from_properties("msg_123", &properties);
        let report = body_coverage_report(&properties, &payloads);

        assert_eq!(report.supported_body_property_count, 0);
        assert_eq!(report.extracted_payload_count, 0);
        assert_eq!(report.fallback_record_count, 1);
        assert_eq!(
            report.status,
            "body_payload_property_absent; supported_body_properties=0"
        );
    }

    #[test]
    fn reports_unusable_body_properties() {
        let mut values = HashMap::new();
        values.insert(
            PR_BODY,
            PropertyValue {
                tag: PR_BODY,
                name: "body_text".to_string(),
                raw: Vec::new(),
                decoded: None,
                status: "selected".to_string(),
            },
        );
        let properties = PropertyContext::from_values(values);
        let payloads = body_payloads_from_properties("msg_123", &properties);
        let report = body_coverage_report(&properties, &payloads);

        assert_eq!(report.supported_body_property_count, 1);
        assert_eq!(report.extracted_payload_count, 0);
        assert_eq!(report.fallback_record_count, 1);
        assert_eq!(
            report.status,
            "body_payload_properties_present_but_unusable; supported_body_properties=1"
        );
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
