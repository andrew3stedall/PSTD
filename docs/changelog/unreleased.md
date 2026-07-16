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
- Bounded Unicode XBLOCK decoding with ordered external child-BID resolution, exact `lcbTotal` assembly, duplicate/internal-child rejection, and DOCX signature validation.
- One validated 11,862-byte `attachment.docx` payload with deterministic archive path and SHA-256 `0c87a742c970907d3b08c73e7834768abadd00fe4f4995a7dd98a206d4c494c0`.
- A permanent Tika attachment fixture workflow that asserts exact metadata, payload bytes, checksum, ZIP CRCs, expected DOCX text, counts, and output bytes.
- Heap-backed Table Context row-matrix resolution through the owning Heap-on-Node allocation, reusing the existing bounded row and recipient projections.
- Direct root-SLBLOCK recipient-table attribution that excludes nested embedded-message tables from the outer message.
- Eight exact Tika recipient records across seven messages: six authoritative SMTP rows and two preserved raw/native rows, including a complete legacy Exchange distinguished name.

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
- Corrected the earlier assumption that `PidTagAttachSize` had to equal the file payload: the fixture preserves 15,503 bytes as attachment metadata while the XBLOCK authoritatively emits 11,862 payload bytes.
- Suppressed unrelated attachment-table fallback rows once the validated filename-bearing Property Context attachment path is selected.
- Rebuilt the root README and current-state documentation so historical milestone/PQ files are no longer presented as the live roadmap.
- Prevented the attachment owner's recursively loaded subnode tree from emitting the same recipient projection twice when attachment presence is inferred from that tree.

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
Recipients: 8
SMTP/raw-native recipients: 6/2
Attachment records: 1
Attachment filename: attachment.docx
PidTagAttachSize: 15503
Attachment payload files/bytes: 1/11862
Attachment SHA-256: 0c87a742c970907d3b08c73e7834768abadd00fe4f4995a7dd98a206d4c494c0
Attachment data NID: 0x0000833f
Attachment data BID: 0x632
XBLOCK child blocks: 2
DOCX ZIP/CRC validation: passed
Expected document text: present
EML files/bytes: 0/0
Attachment JSONL bytes: 643
Recipient JSONL bytes: 2418
Extraction TAR bytes: 202752
Total output bytes: 241579
```

The attachment belongs to `msg_c6163b9157944cc9`. Its validated Property Context resolves through a Unicode SLBLOCK to the internal XBLOCK at BID `0x632`; the XBLOCK resolves two ordered external child blocks and emits one valid DOCX file. The same message now emits its directly owned To recipient, while a nested method-`5` embedded-message recipient table remains excluded.

## In progress

- Validate Date and required header evidence for the Tika attachment message and assemble its first deterministic `multipart/mixed` EML.

## Known limitations

- PSTD is not yet a generally compatible PST converter or PST-to-EML tool.
- The Tika fixture emits recipients and a DOCX payload but still emits no EML files.
- Embedded-message method `5` extraction remains deferred.
- ANSI, uncommon, corrupt, embedded-message, and broad MAPI-layout coverage remain incomplete.
- Non-ASCII RFC 2047 header encoding remains incomplete.
- Downstream Snowflake, UI, search, semantic search, graph, and LLM/RAG work remains parked.

## Removed or superseded

- The earlier assumption that completing M1-M25 made the extraction engine release-complete.
- The PQ-cycle roadmap as the default operating model after the validated parser foundation reached PQ74.
- Stale documentation that described recipient complete-record publication or first readable EML assembly as unfinished.
- Raw `text/rtf` as the preferred rich EML alternative for the current HTML-derived fixture body.
- The earlier evidence blocker that no approved attachment-bearing PST was available.
- The assumption that the attachment file payload must be padded or truncated to the 15,503-byte `PidTagAttachSize` value.
