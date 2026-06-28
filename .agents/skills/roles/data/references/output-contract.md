# PSTD v1 Output Contract Reference

## Purpose

Define the stable output shape that PSTD planning and implementation should preserve.

The v1 command is named `pstd`. The v1 canonical output is structured TAR plus JSONL metadata. EML is not the default output format.

## Output root shape

```text
<output-root>/
  run_summary.json
  progress.jsonl
  checkpoint.sqlite
  archives/
    <pst-id>_000001.tar
  logs/
    run_errors.jsonl
```

## TAR shard shape

```text
_pstfast/
  summary.json
  manifest.jsonl
  errors.jsonl
  folder_inventory.jsonl
  extraction_warnings.jsonl
  run_config.json

data/
  messages.jsonl
  recipients.jsonl
  message_references.jsonl
  bodies.jsonl
  attachments.jsonl
  folders.jsonl
  selected_mapi_properties.jsonl

bodies/
  <message-key>.txt
  <message-key>.html

attachments/
  <message-key>/
    <attachment-key>_<safe-filename>
```

## Stable IDs

Preserve stable keys for future joins, search, tagging, review, downloads, and graph work:

- `run_id`
- `pst_id`
- `folder_key`
- `message_key`
- `recipient_key`
- `body_key`
- `attachment_key`
- `reference_key`

## Required message metadata

Each extracted message should preserve, where available:

- Source PST path.
- Source folder path.
- Subject.
- Sender fields.
- Recipients: to, cc, bcc, reply-to.
- Sent and received times.
- Internet message ID.
- In-reply-to and references headers.
- Conversation fields.
- Raw transport headers when available.
- Attachment count.
- Extraction status.
- Error references.

## Body outputs

- Preserve plain text body when available.
- Preserve HTML body when available.
- Store body content as files inside the TAR.
- Record body path, encoding, size, hash, and status in `bodies.jsonl`.
- Do not invent missing body representations.

## Attachment outputs

- Store attachments as files inside the TAR.
- Do not store attachment bytes inside JSON records.
- Use safe deterministic filenames.
- Preserve original filename in metadata.
- Record skipped or incomplete attachments in `errors.jsonl`.

## Manifest outputs

`manifest.jsonl` is the per-archive index. It should describe generated artefacts, statuses, hashes, sizes, and output paths.

## Error outputs

`errors.jsonl` is append-friendly and should contain one JSON object per warning or issue.

## Stability rules

- Output paths should be deterministic for the same source data and configuration.
- Message keys should avoid collisions.
- Metadata field names should remain stable once released.
- Future Snowflake loading should not require reparsing raw PST files when v1 outputs exist.
