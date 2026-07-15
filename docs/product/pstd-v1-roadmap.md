# PSTD Roadmap

_Last reviewed: 15 July 2026._

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

Complete. The public fixture emits one 1,175-byte `multipart/alternative` EML containing ordered `text/plain` and `text/rtf` parts while preserving sender, recipients, subject, Date, and Message-ID.

### Readable HTML body

Complete. The validated 320-byte `\fromhtml1` RTF produces one 95-byte standalone HTML document for message `msg_ad9f58792ae34dfc` containing bold and blue-text markup with no raw RTF control data.

The milestone preserved one message, two body payload records, four recipients, zero attachments, one 1,175-byte EML, and one 320-byte standalone RTF while increasing combined observable EML, RTF, and HTML output from 1,495 to 1,590 bytes.

## Current milestone

### Emit plain-text and HTML EML alternatives

Replace the EML's `text/rtf` alternative with the newly validated `text/html` representation while retaining the existing `text/plain` part.

Acceptance boundary:

- reuse the validated HTML recovery path rather than reinterpreting RTF independently;
- emit one deterministic `multipart/alternative` EML with ordered `text/plain` and `text/html` parts;
- preserve sender, To, Cc, Subject, Date, and Message-ID;
- preserve the known plain-text body and recovered bold and blue-text HTML markup;
- emit no raw RTF controls in the EML body;
- fail closed when HTML recovery is unavailable, malformed, or unsafe;
- record the exact EML byte count;
- pass focused regression tests and all exact-head fixture workflows.

Attachment work remains deferred because the current public fixture contains zero attachment candidates. Adding attachment abstractions against it would not produce observable attachment data.

## Following decision point

After HTML-backed EML assembly, obtain or add an approved fixture containing at least one real attachment before extending attachment extraction. Do not create attachment-only infrastructure without observable fixture evidence.

These are evidence-led candidates, not a fixed queue.

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
