from pathlib import Path

# Touching this file intentionally triggers the branch-only patch workflow.
path = Path("src/bin/pstd-eml.rs")
text = path.read_text()

replacements = [
    (
        "use std::collections::BTreeMap;",
        "use std::collections::{BTreeMap, BTreeSet};",
    ),
    (
        "    let attachments = attachments_by_message(&metadata.attachment_payloads);\n    let mut emitted = 0usize;",
        "    let attachments = attachments_by_message(&metadata.attachment_payloads);\n    let embedded_messages = embedded_message_keys(&metadata.attachment_payloads);\n    let mut emitted = 0usize;",
    ),
    (
        "        let Some(eml) = build_eml(message, message_recipients, body, message_attachments) else {",
        "        let Some(eml) = build_eml_with_plain_text_policy(\n            message,\n            message_recipients,\n            body,\n            message_attachments,\n            embedded_messages.contains(&message.message_key),\n        ) else {",
    ),
    (
        "fn bodies_by_message(payloads: &[BodyPayload]) -> BTreeMap<String, MessageBodies> {",
        "fn embedded_message_keys(payloads: &[AttachmentPayload]) -> BTreeSet<String> {\n    payloads\n        .iter()\n        .filter_map(|payload| payload.record.embedded_message_key.clone())\n        .collect()\n}\n\nfn bodies_by_message(payloads: &[BodyPayload]) -> BTreeMap<String, MessageBodies> {",
    ),
    (
        "fn build_eml(\n    message: &MessageRecord,\n    recipients: &[RecipientRecord],\n    bodies: &MessageBodies,\n    attachments: &[AttachmentPayload],\n) -> Option<Vec<u8>> {",
        "fn build_eml(\n    message: &MessageRecord,\n    recipients: &[RecipientRecord],\n    bodies: &MessageBodies,\n    attachments: &[AttachmentPayload],\n) -> Option<Vec<u8>> {\n    build_eml_with_plain_text_policy(message, recipients, bodies, attachments, false)\n}\n\nfn build_eml_with_plain_text_policy(\n    message: &MessageRecord,\n    recipients: &[RecipientRecord],\n    bodies: &MessageBodies,\n    attachments: &[AttachmentPayload],\n    allow_plain_text_only: bool,\n) -> Option<Vec<u8>> {",
    ),
    (
        "        } else {\n            push_header(&mut eml, \"Content-Type\", \"text/plain; charset=utf-8\");",
        "        } else {\n            if !allow_plain_text_only {\n                return None;\n            }\n            push_header(&mut eml, \"Content-Type\", \"text/plain; charset=utf-8\");",
    ),
    (
        "        let eml = build_eml(&message(), &[recipient(0, \"to\")], &bodies, &[]).unwrap();",
        "        assert!(build_eml(&message(), &[recipient(0, \"to\")], &bodies, &[]).is_none());\n        let eml = build_eml_with_plain_text_policy(\n            &message(),\n            &[recipient(0, \"to\")],\n            &bodies,\n            &[],\n            true,\n        )\n        .unwrap();",
    ),
]

for old, new in replacements:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"expected exactly one match, found {count}: {old[:80]!r}")
    text = text.replace(old, new)

path.write_text(text)
