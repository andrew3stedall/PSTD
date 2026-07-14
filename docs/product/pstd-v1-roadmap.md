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

Complete. This lane progressed from the first real semantic recipient property to:

- recipient roles;
- display-name and address string references;
- heap-resident string decoding;
- end-to-end recipient identity projection;
- row-aligned role/name/address records;
- address-property selection and address-kind classification;
- complete recipient records retaining role, display name, address, and address kind;
- one production public-fixture execution publishing all four complete records;
- four actual `RecipientRecord` JSONL rows attached to the extracted message.

The public fixture now emits:

```text
To: Recipient 1 <to1@domain.com>
To: Recipient 2 <to2@domain.com>
Cc: Recipient 3 <cc1@domain.com>
Cc: Recipient 4 <cc2@domain.com>
```

The exact run preserved one message, two body payload records, zero attachments, and zero EML files while increasing output from 39,622 to 40,722 bytes. The four recipient records account for the 1,100-byte increase.

## Current milestone

### First complete readable EML

The next milestone must assemble one real `.eml` file from data already validated on the public fixture. It must not introduce another recipient transport, projection, formatter, or diagnostic layer.

Acceptance boundary:

- emit one EML artifact for the one extracted fixture message;
- use the validated subject and sender fields;
- generate `To` and `Cc` headers from the four structured recipient records;
- include the existing plain-text body payload;
- preserve native recipient-address semantics internally while rendering usable RFC-style address headers;
- use deterministic CRLF line endings and header/body separation;
- fail closed when required message, recipient, or body evidence is unavailable or ambiguous;
- add focused serialization tests and a public-fixture assertion over the emitted EML;
- preserve existing message, body, recipient, and attachment outputs;
- pass full CI on the exact merge head.

HTML/RTF multipart selection and attachments are outside this first EML slice unless strictly required to emit the fixture's readable plain-text message.

## Following decision point

After the first EML exists, review the full fixture artifact and choose the largest remaining gap preventing broad conversion coverage. Likely candidates include:

- preferred body selection and encoding fidelity;
- HTML and RTF normalization or multipart alternatives;
- attachment table and payload extraction;
- embedded-message extraction;
- broader fixture validation and malformed-input coverage.

These are candidates, not a fixed queue. The artifact must determine the order.

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

The following remain intentionally outside the active extraction lane:

1. Snowflake ingestion.
2. Search and review web application.
3. Semantic search, embeddings, tagging, graph, and LLM/RAG workflows.
4. Distributed orchestration beyond the current local/Docker batch model.

EML assembly is now active. Exact-preservation policy and large-corpus hardening remain later concerns after the first complete readable message is emitted.
