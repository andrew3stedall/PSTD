# PSTD Roadmap

_Last reviewed: 16 July 2026._

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

Complete. Three pinned public PST fixtures now provide evidence for attachment extraction, multiple messages and folders, body-representation selection, appointments, recurrence exceptions, contacts, distribution lists, and legacy Exchange address handling. Exact provenance, sizes, SHA-256 hashes, and expected upstream evidence are documented in [Upstream PST Fixture Corpus](../operations/upstream-pst-fixture-corpus.md).

### First real attachment filename

Complete. `tika-testPST.pst` now emits one metadata-only attachment record for message `msg_c6163b9157944cc9`:

```text
filename: attachment.docx
declared size: 15,503 bytes
method: 1 (by value)
payload bytes: unresolved
```

The owning message now reports `has_attachments=true` and `attachment_count=1`. The fixture preserves seven messages, eight body records, zero recipients, zero attachment payload files, and zero EML files while increasing attachment records from zero to one. Exact before-versus-after evidence is recorded in [Vertical 29](../operations/vertical-29-expose-docx-attachment-filename.md).

## Current milestone

### Resolve the DOCX attachment payload

The `PidTagAttachDataBinary` value in the validated attachment Property Context is the four-byte value `3f830000`. It is an HNID/reference, not the attachment itself. The next milestone must resolve that reference through existing validated Heap-on-Node/subnode components and emit the real DOCX payload.

Acceptance boundary:

- resolve the reference without scanning for ZIP signatures or guessing from nearby bytes;
- tie the payload unambiguously to `msg_c6163b9157944cc9` and `attachment.docx`;
- require exactly 15,503 payload bytes, matching `PidTagAttachSize`;
- verify the DOCX ZIP signature and calculate a deterministic SHA-256 checksum;
- update the existing attachment record from metadata-only to extracted;
- emit one attachment payload file at its deterministic archive path;
- preserve seven messages, eight body records, zero recipients, and zero Tika-fixture EML files unless independently eligible;
- report exact before-versus-after structured-output, attachment-payload, TAR, EML, and total-output byte counts;
- keep embedded-message method `5` handling out of this by-value payload milestone.

## Following fixture sequence

After the DOCX payload is validated:

1. assemble a deterministic `multipart/mixed` EML when the Tika message has independently validated recipients and body alternatives;
2. recover the method-`5` embedded message as a separate object and attachment path;
3. validate multiple messages, folders, Unicode names, and legacy Exchange address preservation on `tika-testPST.pst`;
4. validate body-form selection with `tika-various-body-types.pst`;
5. validate appointments and recurrence exceptions with `java-libpst-dist-list.pst`;
6. validate contacts and distribution-list entries without forcing them through the normal email path;
7. create a controlled synthetic fixture for true X.400, because the public Exchange legacy DN is X.500-style/`EX`, not a true X.400 O/R address.

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
