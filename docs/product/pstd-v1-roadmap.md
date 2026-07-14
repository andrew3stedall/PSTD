# PSTD Roadmap

_Last reviewed: 14 July 2026._

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
- Treat EML generation as a later assembly layer over reliable extracted data, not as a substitute for parser coverage.

## Completed foundation

### M1-M25: product and operating foundation

Complete. This lane delivered the Rust CLI, Python wrapper, Docker packaging, structured TAR/JSONL output, stable IDs, bodies/attachment record foundations, batch/resume support, diagnostics, fixture workflows, and operator handoff.

### PQ1-PQ74: validated parser and Table Context foundation

Complete. This lane corrected PST traversal, identified real folder/message candidates, improved property and body extraction, resolved Heap-on-Node/BTH/subnode/Table Context structures, validated row addressing and transport, decoded supported fixed-width MAPI values, and integrated bounded diagnostics.

### Vertical 1-16: recipient extraction and production evidence

Complete through production reporting. This lane progressed from the first real semantic row property to:

- recipient roles;
- display-name and address string references;
- heap-resident string decoding;
- end-to-end recipient identity projection;
- row-aligned role/name/address records;
- address-property selection and address-kind classification;
- complete recipient records retaining role, display name, address, and address kind;
- one production public-fixture execution publishing all four complete records.

The public fixture now exposes:

```text
To: Recipient 1 <to1@domain.com>
To: Recipient 2 <to2@domain.com>
Cc: Recipient 3 <cc1@domain.com>
Cc: Recipient 4 <cc2@domain.com>
```

The current output remains diagnostic evidence rather than message-model recipient fields.

## Current milestone

### Vertical 17: attach complete recipients to the extracted message

The next milestone must consume the already validated complete-recipient projection and place the four records on the extracted message model. It must not introduce another standalone projection, formatter, diagnostic, or transport wrapper.

Acceptance boundary:

- role, display name, address, and address kind are available on the message result;
- row order and To/Cc meaning remain unchanged;
- incomplete combined evidence yields no partial message recipients;
- the public fixture emits four structured message recipients in one run;
- existing message, body, attachment, and output behaviour does not regress;
- full CI and public-fixture evidence pass on the exact head.

Where the existing serializer boundary permits without widening the slice materially, the same validated records should feed EML `To` and `Cc` headers. Otherwise EML header emission is the immediately following milestone.

## Following decision point

After recipients are attached to the message model, review the full fixture artifact and select the largest remaining gap preventing a reconstructable email. Likely candidates include:

- EML header and file emission;
- preferred body selection and encoding fidelity;
- HTML/RTF normalization;
- attachment table and payload extraction;
- embedded-message extraction.

These are candidates, not a fixed queue. The artifact must determine the order.

## Completion definition for reliable extraction

PSTD should not be described as conversion-complete until a representative fixture corpus demonstrates, with explicit completeness statuses:

- folder hierarchy preservation;
- message discovery without false positives;
- subject, sender, dates, identifiers, and transport headers where present;
- To/Cc/Bcc recipients with names and usable addresses;
- plain text, HTML, and RTF handling appropriate to the source;
- attachment metadata and bytes, including explicit handling for embedded messages;
- deterministic structured output suitable for EML assembly;
- corruption and unsupported-layout behaviour that fails closed rather than guessing;
- no regressions across the approved fixture set.

## Deferred roadmap

The following remain intentionally outside the active extraction lane:

1. Snowflake ingestion.
2. Search and review web application.
3. Semantic search, embeddings, tagging, graph, and LLM/RAG workflows.
4. Distributed orchestration beyond the current local/Docker batch model.

EML assembly is no longer treated as generally deferred: it should begin incrementally once validated message fields are attached to the message model, while exact-preservation policy remains a later hardening concern.