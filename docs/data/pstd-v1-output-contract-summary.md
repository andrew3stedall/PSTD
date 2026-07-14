# PSTD Structured Output Contract

_Last reviewed: 14 July 2026._

## Purpose

Define the canonical structured output boundary and distinguish the intended record families from the extraction coverage currently proven on real fixtures.

PSTD does not currently use EML as its canonical output. It emits deterministic TAR shards and JSONL records so extracted evidence can be validated, joined, reviewed, and later assembled into EML or loaded downstream.

## Contract versus capability

The existence of a record type or archive path means the output layer supports that shape. It does not mean every source field is currently recovered from every PST.

| Record family | Contract state | Current extraction maturity |
|---|---|---|
| Run, progress, manifest, error, and summary records | Implemented | Operational |
| Folder records and inventory | Implemented | Public fixture produces 11 folders |
| Message records | Implemented | Public fixture produces one true/extracted message; metadata coverage remains incomplete |
| Selected MAPI property records | Implemented | Public fixture produces 16 selected and 19 unknown properties |
| Body records and raw body artefacts | Implemented | Public fixture recovers two body payloads; format coverage remains incomplete |
| Recipient records | Implemented as a contract | Validated role/name/address evidence exists, but complete records are not yet published through one production fixture execution |
| Message-reference records | Implemented as a contract | Coverage is not yet sufficient to claim complete threading fidelity |
| Attachment records and raw attachment artefacts | Implemented as a contract | Public fixture currently emits zero attachments |
| EML | Not canonical and not implemented | Deferred until extracted data is sufficiently complete |

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

Message records should retain source identity, folder relationship, item type, subject/sender/date identifiers where validated, body and attachment indicators, threading fields, transport headers, and per-domain completeness statuses.

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

Attachment records should preserve known metadata even when payload bytes are unavailable, empty, unsupported, or deferred. Raw extracted bytes belong in TAR entries, not base64 JSON.

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

EML generation is a future assembly layer. It should be introduced only after the structured records reliably contain the metadata, recipients, headers, body forms, and attachment bytes required for the intended fidelity level. Exact preservation may require additional raw-property or MIME evidence beyond the current contract.

## Downstream boundary

Snowflake, search, UI, tagging, graph, and LLM/RAG systems should consume this contract. They must not compensate for missing extraction by reparsing source PST files or silently inventing absent values.

The Rust source record types under `src/output/` are the implementation authority when this summary and code diverge. Any contract change must update both the code and this document.
