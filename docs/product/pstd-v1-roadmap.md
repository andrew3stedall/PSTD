# PSTD Roadmap

_Last reviewed: 21 July 2026._

## Objective

Deliver reliable PST email extraction before investing in downstream storage or user-interface systems. Progress is measured by new, correct, observable data extracted from approved PST fixtures while preserving bounded and fail-closed behaviour.

## Roadmap principles

- Prioritise end-to-end extraction capability over parser infrastructure for its own sake.
- Implement the smallest coherent vertical slice that exposes new behaviour.
- Prefer additional producer and layout evidence over abstractions that do not change observable extraction.
- Reuse validated parser components and avoid duplicate interpretations of the same bytes.
- Fail closed when bounds, row counts, property identity, types, references, encodings, ownership, or recursion do not validate.
- Preserve existing extraction behaviour and add exact regression tests for every new path.
- Re-run all approved fixtures after every milestone and revise the next milestone from the artifacts.
- Keep Snowflake, UI, search, analytics, semantic search, and graph work parked.
- Treat EML generation as an assembly layer over validated extracted data, not as a substitute for parser coverage.
- Treat contacts, appointments, tasks, journals, and distribution lists as typed non-mail objects rather than forcing them into EML.

## Completed foundation

### M1-M25: product and operating foundation

Complete. This lane delivered the Rust CLI, Python wrapper, Docker packaging, structured TAR/JSONL output, stable IDs, bodies/attachment record foundations, batch/resume support, diagnostics, fixture workflows, and operator handoff.

### PQ1-PQ74: validated parser and Table Context foundation

Complete. This lane corrected PST traversal, identified real folder/message candidates, improved property and body extraction, resolved Heap-on-Node/BTH/subnode/Table Context structures, validated row addressing and transport, decoded supported fixed-width MAPI values, and integrated bounded diagnostics.

### Original public fixture email path

Complete for the approved fixture. It emits four exact To/Cc recipients, validated plain text and HTML recovered from RTF, and one deterministic 956-byte `multipart/alternative` EML with sender, recipients, subject, Date, and Message-ID.

### Tika attachment and embedded-message path

Complete through Vertical 38:

- one exact 11,862-byte DOCX by-value attachment with preserved 15,503-byte source metadata;
- eight messages including one separately linked method-`5` child;
- nine directly owned recipient records;
- eight exact folder records and seven exact top-level physical message owners;
- independent body-form admission with invalid four-byte binary locators represented as unavailable;
- one deterministic 17,035-byte parent `multipart/mixed` EML;
- one deterministic 453-byte child `text/plain` EML;
- one byte-identical 453-byte method-`5` `message/rfc822` payload;
- fail-closed rejection of missing, mismatched, duplicate, nested, ambiguous, and unsafe child-EML candidates.

### ANSI header diagnostics

Complete in Vertical 39 / PR #473. Version-14 and version-15 root offsets and crypt-method locations are decoded with ANSI-specific widths and offsets. These values are diagnostic only. ANSI roots cannot authorize page traversal or extraction.

## Current milestone

### Qualify an additional Unicode producer fixture

Qualify the public libyal version-23 `pst/outlook.pst` candidate before changing extraction code.

Admission requires:

- immutable upstream repository revision and exact fixture path;
- confirmed redistribution basis;
- exact byte length and SHA-256;
- exact header signature, NDB version, crypt method, and classification;
- deterministic inspect and extract exit behaviour;
- exact counts for folders, typed objects, messages, recipients, body forms, attachments, and EML;
- a complete list of unavailable, encrypted, ambiguous, corrupt, or unsupported records;
- repeated-run byte identity for structured output and any generated EML;
- no regression in existing original-fixture or Tika contracts.

After admission, implement only the smallest coherent vertical that produces new observable behaviour. Priority is:

1. a newly discovered or newly owned email;
2. a new by-value attachment method, format, or storage layout;
3. independently valid plain, HTML, or RTF body selection;
4. additional To, Cc, or Bcc evidence;
5. inline attachment and Content-ID evidence;
6. typed non-mail classification.

Parser rules must not be relaxed merely to make the fixture pass.

## Following compatibility sequence

After the additional Unicode producer baseline:

1. broaden by-value attachment formats and data-tree layouts;
2. validate inline attachments and Content-ID references against exact HTML evidence;
3. add authoritative Exchange-to-SMTP resolution only from validated mapping evidence;
4. implement bounded nested embedded-message recursion with explicit depth and ownership limits;
5. add deterministic corrupt, truncated, duplicate, cross-scope, and ambiguous fixture cases;
6. harden large-file performance and memory limits;
7. expose a narrow stable Rust API after extraction records and diagnostics are sufficiently stable;
8. return to ANSI traversal only when Unicode corpus breadth and fail-closed evidence are materially stronger.

## Stable Rust API boundary

The eventual public API should remain narrower than the internal parser modules. It should support:

- opening a PST with explicit parser limits;
- iterating typed mail and non-mail objects;
- reading message, recipient, body, and attachment records;
- generating deterministic EML from admitted evidence;
- reading diagnostics and completeness statuses;
- distinguishing unavailable, unsupported, corrupt, encrypted, and ambiguous evidence.

Internal page, heap, BTH, and Table Context implementations must not become the de facto compatibility contract.

## Completion definition for reliable extraction

PSTD should not be described as conversion-complete or production-ready until a representative fixture corpus demonstrates, with explicit completeness statuses:

- Unicode and ANSI header classification and supported traversal boundaries;
- folder hierarchy preservation;
- message discovery without false positives or silent omissions;
- subject, sender, dates, identifiers, and transport headers where present;
- To/Cc/Bcc recipients with names and usable addresses where authoritative;
- plain text, HTML, and RTF handling appropriate to the source;
- by-value, embedded, inline, and nested attachment handling within explicit limits;
- typed classification of contacts, appointments, tasks, journals, and distribution lists;
- deterministic structured output and EML assembly;
- malformed, encrypted, corrupt, unsupported, and ambiguous behaviour that fails closed;
- no regressions across the approved fixture set;
- documented performance and memory limits.

## Deferred roadmap

1. Snowflake ingestion.
2. Search and review web application.
3. Semantic search, embeddings, tagging, graph, and LLM/RAG workflows.
4. Distributed orchestration beyond the current local/Docker batch model.

Exact-preservation policy and large-corpus hardening remain later concerns after the readable-message path covers more producers, body forms, attachment layouts, and typed non-mail objects.
