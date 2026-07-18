from pathlib import Path

path = Path("src/bin/pstd-eml.rs")
text = path.read_text()

replacements = [
    (
        "use pstd::output::metadata::{MessageRecord, RecipientRecord};",
        "use pstd::output::metadata::{AttachmentRecord, MessageRecord, RecipientRecord};",
    ),
    (
        "    let embedded_messages = embedded_message_keys(&metadata.attachment_payloads);",
        "    let embedded_messages = embedded_message_keys(&metadata.attachments);",
    ),
    (
        "fn embedded_message_keys(payloads: &[AttachmentPayload]) -> BTreeSet<String> {\n    payloads\n        .iter()\n        .filter_map(|payload| payload.record.embedded_message_key.clone())\n        .collect()\n}",
        "fn embedded_message_keys(records: &[AttachmentRecord]) -> BTreeSet<String> {\n    records\n        .iter()\n        .filter_map(|record| record.embedded_message_key.clone())\n        .collect()\n}",
    ),
]

for old, new in replacements:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"expected exactly one match, found {count}: {old[:100]!r}")
    text = text.replace(old, new)

path.write_text(text)
