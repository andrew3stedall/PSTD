# PSTD Roadmap

_Last reviewed: 18 July 2026._

## Objective

Deliver reliable PST email extraction before investing in downstream storage or user-interface systems. Progress is measured by new, correct, observable data extracted from approved PST fixtures while preserving bounded and fail-closed behaviour.

## Roadmap principles

- Prioritise end-to-end extraction capability over parser infrastructure for its own sake.
- Implement the smallest coherent vertical slice that exposes new behaviour.
- Reuse validated parser components and avoid duplicate interpretations of the same bytes.
- Fail closed when bounds, row counts, property identity, types, references, or encodings do not validate.
- Preserve existing extraction behaviour and add regression tests for every new path.
- Re-run the public fixture after every milestone and revise the next milestone from the artifact.
- Keep Snowflake, UI, search, analytics, semantic search, and graph work parked.
- Treat EML generation as an assembly layer over validated extracted data, not as a substitute for parser coverage.

## Completed foundation

### M1-M25: product and operating foundation

Complete. This lane delivered the Rust CLI, Python wrapper, Docker packaging, structured TAR/JSONL output, stable IDs, bodies/attachment record foundations, batch/resume support, diagnostics, fixture workflows, and operator handoff.

### PQ1-PQ74: validated parser and Table Context foundation

Complete. This lane corrected PST traversal, identified real folder/message candidates, improved property and body extraction, resolved Heap-on-Node/BTH/subnode/Table Context structures, validated row addressing and transport, decoded supported fixed-width MAPI values, and integrated bounded diagnostics.

### Vertical recipient lane: roles through structured output

Complete. The original public fixture emits four actual `RecipientRecord` rows:

```text
To: Recipient 1 <to1@domain.com>
To: Recipient 2 <to2@domain.com>
Cc: Recipient 3 <cc1@domain.com>
Cc: Recipient 4 <cc2@domain.com>
```

The exact run preserved one message, two body payload records, zero attachments, and increased structured output from 39,622 to 40,722 bytes.

### First complete readable EML and Date header

Complete. The original public fixture emits one deterministic CRLF `.eml` assembled from validated sender, subject, recipient, plain-text-body, and transport Date data.

### Readable RTF body

Complete. The original fixture's 336-byte `PidTagRtfCompressed` value produces one validated 320-byte standalone RTF document for message `msg_ad9f58792ae34dfc`.

### Multipart readable EML

Complete. The original public fixture emitted one 1,175-byte `multipart/alternative` EML containing ordered `text/plain` and `text/rtf` parts while preserving sender, recipients, subject, Date, and Message-ID.

### Readable HTML body

Complete. The validated 320-byte `\fromhtml1` RTF produces one 95-byte standalone HTML document for message `msg_ad9f58792ae34dfc` containing bold and blue-text markup with no raw RTF control data.

### Plain-text and HTML EML alternatives

Complete. The original public fixture now emits one deterministic 956-byte `multipart/alternative` EML with ordered `text/plain` and `text/html` parts. It preserves the validated sender, To, Cc, Subject, Date, and Message-ID values and contains no raw RTF MIME part.

### Upstream fixture corpus

Complete. Three pinned public PST fixtures provide evidence for attachment extraction, multiple messages and folders, body-representation selection, appointments, recurrence exceptions, contacts, distribution lists, and legacy Exchange address handling. Exact provenance and expected evidence are documented in [Upstream PST Fixture Corpus](../operations/upstream-pst-fixture-corpus.md).

### First real attachment filename

Complete. `tika-testPST.pst` emits one metadata-only attachment record for `msg_c6163b9157944cc9`: `attachment.docx`, `PidTagAttachSize` 15,503 bytes, method 1. Exact evidence is recorded in [Vertical 29](../operations/vertical-29-expose-docx-attachment-filename.md).

### DOCX attachment data reference

Complete. The attachment's validated raw HNID `3f830000` resolves through the message's Unicode SLBLOCK:

```text
data NID:      0x0000833f
resolved BID:  0x632
payload bytes: 0
```

The mapping affects one message and one attachment record. BID `0x632` is internal and is not emitted as DOCX content. Exact evidence is recorded in [Vertical 30](../operations/vertical-30-resolve-docx-attachment-data-reference.md).

### DOCX attachment payload

Complete. The internal Unicode XBLOCK at BID `0x632` resolves two ordered external child blocks and emits one valid DOCX payload:

```text
filename:                 attachment.docx
XBLOCK payload bytes:     11,862
PidTagAttachSize:          15,503
SHA-256:                   0c87a742c970907d3b08c73e7834768abadd00fe4f4995a7dd98a206d4c494c0
ZIP signature:             50 4b 03 04
DOCX CRC validation:       passed
expected document text:    present
```

The differing size values are preserved rather than forced to agree: the XBLOCK `lcbTotal` is the authoritative payload length, while `PidTagAttachSize` remains source metadata. The fixture preserves seven messages, eight body records, zero recipients, one attachment record, one 11,862-byte attachment payload, and zero EML files. Exact evidence is recorded in [Vertical 31](../operations/vertical-31-emit-docx-attachment-payload.md).

### Heap-backed Tika recipient tables

Complete in PR #452. PSTD now resolves Table Context row matrices stored in the owning Heap-on-Node allocation, attributes only direct NID type `0x12` tables to each message, and emits eight recipients across all seven Tika messages.

Six rows carry authoritative SMTP values. Two rows deliberately preserve native/raw evidence: one complete legacy Exchange distinguished name and the attachment owner's `PidTagEmailAddress`, for which no authoritative SMTP projection is published. The existing DOCX payload is unchanged. Exact evidence is recorded in [Vertical 32](../operations/vertical-32-emit-tika-heap-backed-recipients.md).

### First Tika attachment EML

Complete in PR #454. `msg_c6163b9157944cc9` now emits one deterministic 17,035-byte `multipart/mixed` EML containing its validated 22-byte UTF-8 plain-text body and exact 11,862-byte `attachment.docx` payload. The Date is derived from the message's bounded `PidTagMessageDeliveryTime` FILETIME because neither a transport Date nor submit time is available. The four-byte `PidTagHtml` value is invalid UTF-8 and is deliberately excluded. The raw native Exchange sender and recipient evidence are preserved without inventing SMTP.

Exact MIME validation confirms CRLF line endings, one plain-text body, one DOCX attachment, registered DOCX content type, stable filename, base64 transport, and byte-identical decoded payload. Structured extraction, TAR, and total extraction-output bytes remain unchanged.

### Method-5 embedded message recovery

Complete in PR #455. The method-`5` Property Context now preserves its PtypObject HNID, parses the exact eight-byte object allocation, requires a normal-message NID, and resolves that NID exactly once inside the outer message's loaded subnode scope. The child is emitted as `msg_0ff529af59d373d5` and linked from attachment ordinal `1` through `embedded_message_key`.

The child owns one raw/native recipient, a 23-byte UTF-8 text body, and four raw `PidTagHtml` bytes. Its subtree is isolated before recipient projection, so none of those values enter the parent. The outer DOCX remains ordinal `0`, and its 17,035-byte EML is unchanged. Exact evidence is recorded in [Vertical 34](../operations/vertical-34-recover-tika-embedded-message.md).

### Recovered child plain-text EML

Complete in PR #457. The linked child now emits one deterministic 453-byte attachmentless `text/plain` EML. Admission is restricted to message keys referenced by authoritative attachment metadata, so unrelated top-level plain-only records remain fail-closed. The child preserves native Exchange addresses, validated headers, exact CRLF body assembly, and SHA-256 `86ffe5567da7aa505b8be16400889170ca583fd247cc0758f00a43c2a8a99420`; its raw `7f 83 00 00` HTML evidence is excluded. Exact evidence is recorded in [Vertical 35](../operations/vertical-35-emit-tika-child-eml.md).

### Method-5 child attachment payload

Complete in PR #461. The exact 453-byte child EML is now published as the existing method-`5` attachment's `message/rfc822` payload. It retains the parent key, attachment key, ordinal, filename, archive path and `embedded_message_key`, and is byte-identical to the standalone child EML. Missing, duplicate, mismatched, nested and unsafe candidates remain fail-closed. Exact evidence is recorded in [Vertical 36](../operations/vertical-36-materialise-method5-eml-payload.md).

## Current milestone

### Lock complete Tika folder and message coverage

The next smallest vertical must:

- assert all folder paths, names, parent relationships and counts in `tika-testPST.pst`;
- attribute all seven top-level messages and the recovered child to the correct owners without false positives;
- lock Unicode names and subjects where present;
- preserve all nine recipient records, including raw/native legacy Exchange evidence;
- retain the exact DOCX, parent EML, child EML and method-`5` payload contracts;
- classify any non-mail or unsupported object explicitly rather than forcing it into EML.

## Following fixture sequence

After complete Tika folder/message validation:

1. validate body-form selection with `tika-various-body-types.pst`;
2. add the first pinned public ANSI PST baseline;
3. validate appointments and recurrence exceptions with `java-libpst-dist-list.pst`;
4. validate contacts and distribution-list entries without forcing them through the normal email path;
5. create a controlled synthetic fixture for true X.400, because the public Exchange legacy DN is X.500-style/`EX`, not a true X.400 O/R address.

## Completion definition for reliable extraction

PSTD should not be described as conversion-complete until a representative fixture corpus demonstrates, with explicit completeness statuses:

- folder hierarchy preservation;
- message discovery without false positives;
- subject, sender, dates, identifiers, and transport headers where present;
- To/Cc/Bcc recipients with names and usable addresses;
- plain text, HTML, and RTF handling appropriate to the source;
- attachment metadata and bytes, including explicit handling for embedded messages;
- deterministic structured output and EML assembly;
- corruption and unsupported-layout behaviour that fails closed rather than guessing;
- no regressions across the approved fixture set.

## Deferred roadmap

1. Snowflake ingestion.
2. Search and review web application.
3. Semantic search, embeddings, tagging, graph, and LLM/RAG workflows.
4. Distributed orchestration beyond the current local/Docker batch model.

Exact-preservation policy and large-corpus hardening remain later concerns after the readable-message path covers more body formats and attachments.
