# PSTD v1 Data and Output Contract Summary

## Purpose

Define the v1 structured archive contract at a level that implementation agents can follow and future Snowflake/search/tagging work can consume.

This summary updates the earlier repo reference from an EML-centred shape toward the v1 structured TAR contract. EML may be generated later, but it is not the default canonical output for v1.

## Default output strategy

```text
PST
  -> Rust extraction engine
  -> structured records
  -> raw body files
  -> raw attachment files
  -> TAR shards
  -> diagnostics and summaries
```

## Output root shape

```text
<output-root>/
  run_summary.json
  progress.jsonl
  checkpoint.sqlite

  archives/
    <pst_id>_000001.tar
    <pst_id>_000002.tar

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
  <message_key>.txt
  <message_key>.html

attachments/
  <message_key>/
    <attachment_key>_<safe_filename>
```

## Stable IDs

Every record should include enough stable IDs for later Snowflake joins, tagging, review, generated email download, and graph construction.

Required key families:

```text
run_id
pst_id
folder_key
message_key
recipient_key
body_key
attachment_key
reference_key
```

Recommended key strategy:

- `pst_id`: deterministic ID from source path plus size/hash metadata where available.
- `folder_key`: `pst_id` plus folder node ID/path identity.
- `message_key`: `pst_id` plus message node ID or equivalent stable PST identity.
- `recipient_key`: `message_key` plus recipient type and ordinal.
- `body_key`: `message_key` plus body type.
- `attachment_key`: `message_key` plus attachment ordinal or attachment node ID.
- `reference_key`: `message_key` plus reference ordinal.

## Required JSONL files

### `data/messages.jsonl`

One row per discovered email-like item.

Important fields:

```text
run_id
pst_id
folder_key
message_key
message_node_id
folder_path
item_type
subject
sender_name
sender_email
sender_raw_address
sender_address_type
sent_at
received_at
created_at
modified_at
transport_message_headers
internet_message_id
in_reply_to_id
conversation_index
conversation_topic
normalized_subject
has_text_body
has_html_body
has_attachments
attachment_count
metadata_status
threading_status
body_status
attachment_status
extraction_status
```

### `data/recipients.jsonl`

One row per recipient.

Important fields:

```text
message_key
recipient_key
recipient_type
display_name
raw_address
address_type
smtp_address
resolution_status
ordinal
```

### `data/message_references.jsonl`

One row per `References` or reply relationship item.

Important fields:

```text
message_key
reference_key
reference_type
referenced_internet_message_id
ordinal
source
```

### `data/bodies.jsonl`

One row per body artefact.

Important fields:

```text
message_key
body_key
body_type
archive_path
encoding
size_bytes
sha256
status
```

### `data/attachments.jsonl`

One row per attachment.

Important fields:

```text
message_key
attachment_key
filename_original
filename_safe
content_type
extension
size_bytes
sha256
is_inline
content_id
ordinal
archive_path
extraction_status
```

### `data/folders.jsonl`

One row per folder.

Important fields:

```text
pst_id
folder_key
parent_folder_key
folder_path
folder_name
folder_node_id
item_count_total
child_folder_count
status
```

### `data/selected_mapi_properties.jsonl`

Optional selected property rows for debug/audit and future enrichment.

Important fields:

```text
message_key
property_id
property_name
property_type
value_summary
source
status
```

### `_pstfast/folder_inventory.jsonl`

One row per folder inventory summary.

Important fields:

```text
pst_id
folder_key
folder_path
item_count_total
item_count_email
item_count_calendar
item_count_contact
item_count_task
item_count_unknown
child_folder_count
inventory_status
```

### `_pstfast/errors.jsonl`

One row per warning/error.

Important fields:

```text
run_id
source_pst
scope
severity
code
message
source_folder_path
message_key
attachment_key
timestamp_utc
recoverable
raw_offset
```

### `_pstfast/manifest.jsonl`

One row per discovered item or output artefact.

Important fields:

```text
run_id
pst_id
message_key
folder_key
artefact_type
archive_path
sha256
size_bytes
status
error_count
```

### `_pstfast/summary.json`

Per-PST summary.

Important fields:

```text
pst_id
source_pst_path
pst_size_bytes
started_at
finished_at
duration_seconds
folders_discovered
messages_discovered
messages_extracted
messages_failed
attachments_extracted
attachments_failed
bytes_read
bytes_written
tar_shards_written
throughput_messages_per_second
throughput_mb_per_second
status
```

## Body and attachment storage rules

- Do not base64 encode attachments into JSON.
- Write attachment bytes as raw TAR entries.
- Preserve text and HTML bodies as files where available.
- Preserve raw transport headers on `data/messages.jsonl` when available.
- Record hashes and byte sizes for future validation.
- Record missing or failed extraction explicitly.

## EML policy

EML is not the v1 canonical output. Future generated download is supported by storing enough structured metadata, body content, headers, and attachments to reconstruct a display-equivalent `.eml` later.

If legal/archive-grade byte-for-byte fidelity becomes required, a later milestone must add raw MIME/EML preservation or richer raw property capture.

## Error and completeness status

Records should distinguish extraction failure from partial extraction.

Recommended statuses:

```text
success
partial_success
failed
missing
unsupported
skipped_unsupported_type
skipped_corrupt
```

Recommended completeness fields:

```text
metadata_status
threading_status
body_status
attachment_status
```

## Future Snowflake readiness

The archive contract should support later loading into tables such as:

```text
EMAIL_MESSAGE
EMAIL_RECIPIENT
EMAIL_MESSAGE_REFERENCE
EMAIL_BODY
EMAIL_ATTACHMENT
EMAIL_FOLDER
EMAIL_ERROR
EMAIL_TAG
```

Future search and graph work should consume these outputs. They should not require reparsing the source PST files.
