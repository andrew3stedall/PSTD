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

Complete. The public fixture emits four actual `RecipientRecord` rows:

```text
To: Recipient 1 <to1@domain.com>
To: Recipient 2 <to2@domain.com>
Cc: Recipient 3 <cc1@domain.com>
Cc: Recipient 4 <cc2@domain.com>
```

The exact run preserved one message, two body payload records, zero attachments, and increased structured output from 39,622 to 40,722 bytes.

### First complete readable EML and Date header

Complete. The public fixture emits one deterministic CRLF `.eml` assembled from validated sender, subject, recipient, plain-text-body, and transport Date data.

### Readable RTF body

Complete. The fixture's 336-byte `PidTagRtfCompressed` value produces one validated 320-byte standalone RTF document for message `msg_ad9f58792ae34dfc` containing:

```text
This line is in bold.
This line is in blue color
```

### Multipart readable EML

Complete. The public fixture emitted one 1,175-byte `multipart/alternative` EML containing ordered `text/plain` and `text/rtf` parts while preserving sender, recipients, subject, Date, and Message-ID.

### Readable HTML body

Complete. The validated 320-byte `\fromhtml1` RTF produces one 95-byte standalone HTML document for message `msg_ad9f58792ae34dfc` containing bold and blue-text markup with no raw RTF control data.

The milestone preserved one message, two body payload records, four recipients, zero attachments, one 1,175-byte EML, and one 320-byte standalone RTF while increasing combined observable EML, RTF, and HTML output from 1,495 to 1,590 bytes.

### Plain-text and HTML EML alternatives

Complete. The public fixture now emits one deterministic 956-byte `multipart/alternative` EML with ordered `text/plain` and `text/html` parts. It preserves the validated sender, To, Cc, Subject, Date, and Message-ID values, includes the known plain body and recovered bold/blue HTML markup, and contains no `text/rtf`, raw `\rtf`, or `\htmltag` content.

The exact fixture result preserves one message, two body payload records, four structured recipients, zero attachments, one 320-byte standalone RTF, and one 95-byte standalone HTML file. Combined observable EML, RTF, and HTML bytes changed from 1,590 to 1,371 because the compact HTML alternative replaced the larger raw RTF MIME representation.

### Upstream fixture corpus

Complete. Three pinned public PST fixtures now provide evidence for attachment extraction, multiple messages and folders, body-representation selection, appointments, recurrence exceptions, contacts, distribution lists, and legacy Exchange address handling. Exact provenance, sizes, SHA-256 hashes, and expected upstream evidence are documented in [Upstream PST Fixture Corpus](../operations/upstream-pst-fixture-corpus.md).

## Current milestone

### Extract the first real attachment field

Use `tests/fixtures/upstream/tika-testPST.pst`, which contains a documented nested `attachment.docx`, multiple messages, multiple folders, Unicode metadata, and a legacy Exchange recipient address.

The smallest acceptable vertical result is one attachment property tied unambiguously to its owning message. Prefer this order:

1. validated attachment filename;
2. validated attachment size;
3. attachment method or MIME type;
4. exact attachment payload bytes;
5. structured attachment output;
6. deterministic `multipart/mixed` EML assembly.

Acceptance boundary:

- reuse the existing validated NDB, subnode, Heap-on-Node, BTH, Property Context, and Table Context components;
- identify the attachment table and owner message without heuristics;
- fail closed if row attribution, subnode resolution, or property type is ambiguous;
- retain the original small fixture as a regression gate;
- add a fixture-specific test or workflow that names `tika-testPST.pst` explicitly;
- report before-versus-after message, body, recipient, attachment, EML, and byte counts;
- do not add attachment abstractions unless they expose a real value from this fixture in the same milestone.

## Following fixture sequence

After the first attachment field and payload are validated:

1. complete attachment-to-EML assembly on `tika-testPST.pst`;
2. validate multiple messages, folders, Unicode names, and legacy Exchange address preservation on the same fixture;
3. validate body-form selection with `tika-various-body-types.pst`;
4. validate appointments and recurrence exceptions with `java-libpst-dist-list.pst`;
5. validate contacts and distribution-list entries without forcing them through the normal email path;
6. create a controlled synthetic fixture for true X.400, because the public Exchange legacy DN is X.500-style/`EX`, not a true X.400 O/R address.

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
