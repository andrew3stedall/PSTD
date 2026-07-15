# Unreleased

_Last reviewed: 15 July 2026._

## Added

### Product foundation

- Rust `pstd` CLI with `inspect`, `extract`, `batch`, and `version` commands.
- Python operator wrapper and Docker packaging.
- Structured TAR/JSONL output, stable identifiers, run summaries, progress logs, batch checkpoints, resume-by-skip behaviour, and operator handoff documentation.
- Folder, message, body, recipient, threading, attachment, selected-property, manifest, error, and summary record foundations.

### Parser and extraction

- Safe PST header/root selection, bounded byte reads, checked arithmetic, BBT/NBT traversal, block and subnode access, depth/cycle guards, Heap-on-Node, BTH, Property Context, and Table Context parsing.
- Public PST progress workflow and deterministic bounded artifacts.
- Table Context descriptor evidence, bitmap mapping, row-payload candidate resolution, direct/ordinal addressing, validated row transport, fixed-width scalar decoding, and production diagnostics through PQ74.
- End-to-end recipient extraction from Table Context rows, including:
  - `PidTagRecipientType` role interpretation;
  - display-name, native email-address, and SMTP-address string resolution;
  - HNID and heap-resident value handling;
  - fail-closed row alignment and table attribution;
  - four structured `RecipientRecord` rows in production output.
- Readable EML assembly from validated message metadata, recipients, transport Date, Message-ID, and body payloads.
- Validated standalone RTF extraction for direct, MELA, and LZFu representations.
- HTML recovery from validated `\fromhtml1` RTF with bounded destination handling.
- Deterministic 956-byte `multipart/alternative` EML output with ordered `text/plain` and `text/html` parts.

## Changed

- Shifted the active development model from milestone/PQ infrastructure work to evidence-led vertical extraction milestones.
- Prioritised extraction correctness and observable email fields over downstream Snowflake, UI, search, analytics, or graph implementation.
- Corrected B-tree page metadata and child-reference traversal.
- Decoded permitted `NDB_CRYPT_PERMUTE` blocks while preserving internal blocks as raw.
- Tightened payload admission so structurally invalid table declarations fail closed.
- Increased selected public-fixture properties from 0 to 16 and reduced unknown properties from 74 to 19.
- Recovered public-fixture text and RTF body payloads and eliminated the former fallback body row.
- Replaced legacy table assumptions with the real TC heap, row-index BTH, subnode-backed row storage, and four validated 52-byte rows.
- Prevented internal LTP row bookkeeping properties from being reported as user-readable fields.
- Preferred authoritative SMTP/native address properties over display-name fallback while retaining display names separately at the complete-record boundary.
- Replaced the public fixture EML's raw `text/rtf` alternative with validated recovered `text/html` while retaining plain text and all validated headers.
- Rebuilt the root README and current-state documentation so historical milestone/PQ files are no longer presented as the live roadmap.

## Current public-fixture result

```text
BBT/NBT entries: 50/63
Folders: 11
True/extracted messages: 1/1
Body payloads: 2
Attachments: 0
Selected/unknown properties: 16/19
Validated Table Context rows: 4 x 52 bytes
Recipient roles: to, to, cc, cc
Recipient display names: Recipient 1..4
Recipient addresses: to1@domain.com, to2@domain.com, cc1@domain.com, cc2@domain.com
Address classification: native_email_address
Structured recipient records: 4
EML files: 1
EML MIME alternatives: text/plain, text/html
EML bytes: 956
Standalone RTF bytes: 320
Standalone HTML bytes: 95
```

The public fixture now produces one readable email containing sender, To/Cc recipients, subject, Date, Message-ID, plain text, and recovered HTML. The generated EML is deterministic CRLF output and contains no raw RTF MIME part.

## In progress

- Broader fixture coverage and attachment extraction remain evidence-blocked until an approved PST containing at least one real attachment or additional message layout is available.

## Known limitations

- PSTD is not yet a generally compatible PST converter or PST-to-EML tool.
- The approved public fixture contains zero attachment candidates, so attachment filename, MIME type, payload, and embedded-message extraction are not yet validated end to end.
- ANSI, uncommon, corrupt, embedded-message, and broad MAPI-layout coverage remain incomplete.
- Non-ASCII RFC 2047 header encoding remains incomplete.
- Downstream Snowflake, UI, search, semantic search, graph, and LLM/RAG work remains parked.

## Removed or superseded

- The earlier assumption that completing M1-M25 made the extraction engine release-complete.
- The PQ-cycle roadmap as the default operating model after the validated parser foundation reached PQ74.
- Stale documentation that described recipient complete-record publication or first readable EML assembly as unfinished.
- Raw `text/rtf` as the preferred rich EML alternative for the current HTML-derived fixture body.
