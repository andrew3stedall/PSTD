# Public and Sanitized Fixture Triage Guide

## Purpose

Provide a safe process for using public or sanitized PST samples to improve parser compatibility without committing mailbox content accidentally.

## What can be used

Use a PST fixture only when at least one of the following is true:

- The file is a public sample with clear redistribution rights.
- The file is synthetic and generated for parser testing.
- The file has been sanitized so that no personal, confidential, credential, financial, health, legal, or customer data remains.

## What must not be committed

Do not commit:

- Real mailbox data.
- Private email bodies.
- Attachments from real mailboxes.
- Batch checkpoints containing local paths or mailbox-derived metadata.
- Extracted TAR archives from real mailboxes.
- Any file where licensing or redistribution rights are unclear.

## Triage workflow

1. Record the fixture source and licensing notes.
2. Run `pstd inspect` and `pstd extract` locally.
3. Review metadata statuses for unsupported subnode layouts, attachment table parse errors, missing payloads, and truncation statuses.
4. Convert each unsupported or partial compatibility category into a focused issue.
5. Add or update synthetic tests that reproduce the smallest relevant layout shape.
6. Only then expand parser logic.

## Compatibility categories

| Category | Meaning | Follow-up |
|---|---|---|
| `table_context_layout` | Existing table parser handled the observed layout. | Keep fixture coverage. |
| `known_child_reference_layout` | Recursive loading followed a known child-reference layout. | Keep depth and duplicate-guard coverage. |
| `unsupported_subnode_layout` | Subnode bytes did not match a supported layout. | Add fixture fingerprint and a focused decoder test. |
| `unparseable_attachment_table` | Attachment table parsing failed. | Record offset and reason before parser expansion. |
| `attachment_rows_without_payloads` | Rows parsed, but payload bytes were not found. | Determine whether payloads are absent, indirect, or in child subnodes. |

## Minimum evidence for a compatibility issue

Include:

- Fixture source or synthetic generator notes.
- PST variant if known.
- Command used.
- Relevant status string.
- Block offset or parse-error offset if available.
- Parse-error reason if available.
- Whether body, attachment, recipient, folder, or table extraction was affected.
- Proposed smallest synthetic test.

## Local validation commands

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo run -- --help
cargo run -- batch --help
cargo run -- inspect --help
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
```

## Rule of thumb

Do not expand parser behaviour from a single opaque failure. First classify it, preserve the diagnostic, add a focused test, and keep the previous fallback status intact.
