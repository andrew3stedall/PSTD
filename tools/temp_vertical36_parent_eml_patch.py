from pathlib import Path

path = Path("src/bin/pstd-eml.rs")
text = path.read_text()

replacements = [
    (
        "    for payload in payloads {\n        grouped\n            .entry(payload.record.message_key.clone())\n            .or_default()\n            .push(payload.clone());\n    }",
        "    for payload in payloads\n        .iter()\n        .filter(|payload| payload.record.attachment_method == Some(1))\n    {\n        grouped\n            .entry(payload.record.message_key.clone())\n            .or_default()\n            .push(payload.clone());\n    }",
    ),
    (
        "    fn attachment(ordinal: usize, bytes: &[u8]) -> AttachmentPayload {",
        "    #[test]\n    fn excludes_embedded_message_payloads_from_parent_mime_assembly() {\n        let by_value = attachment(0, b\"docx\");\n        let mut embedded = attachment(1, b\"child eml\");\n        embedded.record.attachment_method = Some(5);\n        embedded.record.content_type = Some(\"message/rfc822\".to_string());\n\n        let grouped = attachments_by_message(&[by_value, embedded]);\n        let payloads = grouped.get(\"message\").unwrap();\n        assert_eq!(payloads.len(), 1);\n        assert_eq!(payloads[0].record.attachment_method, Some(1));\n    }\n\n    fn attachment(ordinal: usize, bytes: &[u8]) -> AttachmentPayload {",
    ),
]

for old, new in replacements:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"expected one source replacement, found {count}: {old[:80]!r}")
    text = text.replace(old, new, 1)

path.write_text(text)
