# Unreleased

_Last reviewed: 16 July 2026._

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
- Three pinned upstream PST fixtures for attachment, multi-message, body-type, calendar, recurrence, contact, distribution-list, and Exchange-address validation.
- Filename-bearing attachment metadata extraction from validated recursive heap Property Contexts.
- Exact attachment data-NID to loaded data-BID resolution through validated Unicode SLBLOCK entries.
- A permanent Tika attachment fixture workflow that asserts exact metadata, references, counts, and output bytes.

## Changed

- Shifted the active development model from milestone/PQ infrastructure work to evidence-led vertical extraction milestones.
- Prioritised extraction correctness and observable email fields over downstream Snowflake, UI, search, analytics, or graph implementation.
- Corrected B-tree page metadata and child-reference traversal.
- Decoded permitted `NDB_CRYPT_PERMUTE` blocks while preserving internal blocks as raw.
- Tightened payload admission so structurally invalid table declarations fail closed.
- Increased selected original-fixture properties from 0 to 16 and reduced unknown properties from 74 to 19.
- Recovered original-fixture text and RTF body payloads and eliminated the former fallback body row.
- Replaced legacy table assumptions with the real TC heap, row-index BTH, subnode-backed row storage, and four validated 52-byte rows.
- Prevented internal LTP row bookkeeping properties from being reported as user-readable fields.
- Preferred authoritative SMTP/native address properties over display-name fallback while retaining display names separately at the complete-record boundary.
- Replaced the original fixture EML's raw `text/rtf` alternative with validated recovered `text/html` while retaining plain text and all validated headers.
- Marked the Tika DOCX-bearing message as attachment-bearing from validated recursive Property Context evidence even though its direct message context omits `PidTagHasAttachments`.
- Resolved the Tika attachment HNID `0x0000833f` to loaded data BID `0x632` without treating the internal block as DOCX bytes.
- Rebuilt the root README and current-state documentation so historical milestone/PQ files are no longer presented as the live roadmap.

## Current original-fixture result

```text
BBT/NBT entries: 50/63
Folders: 11
True/extracted messages: 1/1
Body payloads: 2
Attachments: 0
Selected/unknown properties: 16/19
Validated Table Context rows: 4 x 52 bytes
Structured recipient records: 4
EML files: 1
EML MIME alternatives: text/plain, text/html
EML bytes: 956
Standalone RTF bytes: 320
Standalone HTML bytes: 95
```

The original public fixture produces one readable email containing sender, To/Cc recipients, subject, Date, Message-ID, plain text, and recovered HTML.

## Current Tika attachment result

```text
Messages: 7
Body records: 8
Body payload files/bytes: 6/252
Recipients: 0
Attachment records: 1
Attachment filename: attachment.docx
Attachment declared size: 15503
Attachment method: 1
Attachment data NID: 0x0000833f
Attachment data BID: 0x632
Attachment payload files/bytes: 0/0
EML files/bytes: 0/0
Attachment JSONL bytes: 648
Extraction TAR bytes: 126464
Total output bytes: 147692
```

The attachment belongs to `msg_c6163b9157944cc9`. Its metadata comes from a validated recursive heap Property Context, and its four-byte `PidTagAttachDataBinary` value now resolves through a complete Unicode SLBLOCK to loaded internal BID `0x632`. The internal block is not reported as payload bytes.

## In progress

- Decode BID `0x632` as an internal data-tree block, resolve its ordered external child blocks, and emit exactly 15,503 DOCX bytes with a verified ZIP signature and SHA-256 checksum.

## Known limitations

- PSTD is not yet a generally compatible PST converter or PST-to-EML tool.
- Attachment data-tree decoding, payload emission, and embedded-message method `5` extraction are not yet complete.
- Recipient extraction remains incomplete on the Tika fixture, so it currently emits no EML files.
- ANSI, uncommon, corrupt, embedded-message, and broad MAPI-layout coverage remain incomplete.
- Non-ASCII RFC 2047 header encoding remains incomplete.
- Downstream Snowflake, UI, search, semantic search, graph, and LLM/RAG work remains parked.

## Removed or superseded

- The earlier assumption that completing M1-M25 made the extraction engine release-complete.
- The PQ-cycle roadmap as the default operating model after the validated parser foundation reached PQ74.
- Stale documentation that described recipient complete-record publication or first readable EML assembly as unfinished.
- Raw `text/rtf` as the preferred rich EML alternative for the current HTML-derived fixture body.
- The earlier evidence blocker that no approved attachment-bearing PST was available.
