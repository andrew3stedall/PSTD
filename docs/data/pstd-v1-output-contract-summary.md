# PSTD Structured Output Contract

_Last reviewed: 17 July 2026._

## Purpose

Define the canonical structured output boundary and distinguish the intended record families from the extraction coverage currently proven on real fixtures.

PSTD does not currently use EML as its canonical output. It emits deterministic TAR shards and JSONL records so extracted evidence can be validated, joined, reviewed, and later assembled into EML or loaded downstream.

## Contract versus capability

The existence of a record type or archive path means the output layer supports that shape. It does not mean every source field is currently recovered from every PST.

| Record family | Contract state | Current extraction maturity |
|---|---|---|
| Run, progress, manifest, error, and summary records | Implemented | Operational |
| Folder records and inventory | Implemented | Public fixture produces 11 folders |
| Message records | Implemented | Original fixture emits one message; Tika emits seven top-level messages plus one separately linked embedded message |
| Selected MAPI property records | Implemented | Public fixture produces 16 selected and 19 unknown properties |
| Body records and raw body artefacts | Implemented | Original fixture recovers two payloads; Tika currently emits ten records and eight payload files, including raw non-markup HTML evidence |
| Recipient records | Implemented | Row-aligned role/name/address records are fixture validated, including direct ownership for one embedded child |
| Message-reference records | Implemented as a contract | Coverage is not yet sufficient to claim complete threading fidelity |
| Attachment records and raw attachment artefacts | Implemented | Tika emits one by-value DOCX payload plus one metadata-only method-`5` record linked to its child message |
| EML | Non-canonical assembly output | Two fixture paths are validated: one 956-byte plain/HTML EML and one 17,035-byte plain/DOCX EML |

## Single-PST output root

```text
<output-root>/
  run_summary.json
  progress.jsonl
  logs/
    run_errors.jsonl
  archives/
    <pst-id>_000001.tar
    <pst-id>_000002.tar
```

## Batch output root

```text
<batch-output-root>/
  batch_summary.json
  batch_checkpoint.jsonl
  batch_progress.jsonl
  <safe-pst-output-dir>/
    run_summary.json
    progress.jsonl
    archives/
      <pst-id>_000001.tar
```

## TAR shard layout

```text
_pstfast/
  summary.json
  manifest.jsonl
  errors.jsonl
  folder_inventory.jsonl
  extraction_warnings.jsonl
  run_config.json

data/
  folders.jsonl
  messages.jsonl
  recipients.jsonl
  message_references.jsonl
  bodies.jsonl
  attachments.jsonl
  selected_mapi_properties.jsonl

bodies/
  <message-key>.txt
  <message-key>.html

attachments/
  <message-key>/
    <attachment-key>_<safe-filename>
```

Files may be absent or empty when the command or source data does not produce that family. Completeness must be represented by statuses and summaries rather than inferred from the directory name alone.

## Stable identifiers

Record families should use deterministic join keys:

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

Keys must derive from stable source identity and ordinal/node information rather than mutable display text.

## Core record expectations

### Messages

Message records should retain source identity, folder relationship, item type, subject/sender/date identifiers where validated, body and attachment indicators, threading fields, transport headers, and per-domain completeness statuses. An embedded child uses its own stable `message_key`, `item_type=embedded_message_metadata`, and source NID identity; it is not folded into the parent record.

### Recipients

Recipient records should retain:

```text
message_key
recipient_key
ordinal
recipient_type
display_name
raw_address
address_type
smtp_address
resolution_status
```

A native `PidTagEmailAddress` value must not be labelled SMTP unless authoritative SMTP evidence exists. Complete records must preserve row alignment and must not combine evidence from separate parser executions.

### Bodies

Body records should identify body type, archive path, encoding, size, hash, and status. Raw body artefacts should be stored as files rather than embedded in JSON.

### Attachments

Attachment records should preserve known metadata even when payload bytes are unavailable, empty, unsupported, or deferred. Raw extracted bytes belong in TAR entries, not base64 JSON. A method-`5` record may carry `embedded_message_key` to link a separately emitted child; that optional field does not imply that an EML payload exists at `archive_path`.

### Errors and completeness

Errors and warnings should carry scope, severity, code, source identity, recoverability, and bounded location evidence where safe.

Recommended statuses include:

```text
success
partial_success
failed
missing
unsupported
unavailable
skipped_unsupported_type
skipped_corrupt
skipped_completed
failed_stopped_early
```

Domain completeness should remain separate:

```text
metadata_status
threading_status
body_status
attachment_status
recipient_status
```

## Batch rules

- Keep discovered, attempted, completed, partial, failed, skipped, and not-run counts separate.
- Preserve append-only checkpoint and progress streams.
- Make reruns deterministic and explicit when outputs are skipped or overwritten.
- Do not report batch success when individual PSTs are partial or failed without preserving those counts.

## Privacy and payload rules

- Never place private PST files in source control or CI artifacts.
- Do not publish complete message bodies or attachment payloads in diagnostic artifacts.
- Do not base64 raw attachment data into JSONL.
- Bound diagnostic strings and sanitise delimiters.
- Preserve hashes and byte counts where available for later verification.

## EML policy

EML generation is a non-canonical assembly layer over validated structured evidence. Current fixture gates cover one plain/HTML and one plain/DOCX message. Attachmentless plain-text-only child assembly, method-`5` payload materialisation, and exact preservation remain incomplete and require their own byte-level contracts.

## Downstream boundary

Snowflake, search, UI, tagging, graph, and LLM/RAG systems should consume this contract. They must not compensate for missing extraction by reparsing source PST files or silently inventing absent values.

The Rust source record types under `src/output/` are the implementation authority when this summary and code diverge. Any contract change must update both the code and this document.
