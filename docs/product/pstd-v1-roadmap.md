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

### Vertical 1-13: recipient extraction

Complete on `main`. This lane progressed from the first real semantic row property to:

- recipient roles;
- display-name and address string references;
- heap-resident string decoding;
- end-to-end recipient identity projection;
- production recipient diagnostics;
- row-aligned role/name records;
- address-property selection and address-kind classification;
- complete recipient records retaining role, display name, address, and address kind.

## Current milestone

### Vertical 14: project complete recipient records in one run

Draft PR #430 is the active implementation. It must independently project `PidTagDisplayName` and the preferred address property from the same validated rows and Table Context heap, then assemble complete records without joining evidence from separate executions.

Acceptance boundary:

- exact same-row and same-heap projection;
- existing validated role projection reused;
- SMTP address preferred over native email address when authoritative and complete;
- separate name and address diagnostics retained;
- no partial records on mismatch or failure;
- full CI and public-fixture evidence on the exact head.

This capability is not part of `main` until the PR is green and merged.

## Next milestone candidate

### Vertical 15: publish complete recipient records through production reporting

After Vertical 14 validates:

1. attach the complete-recipient projection to the production Table Context reporting path;
2. emit one bounded complete-record diagnostic from the public fixture;
3. confirm all four rows retain role, name, address, and address kind in one execution;
4. preserve explicit unavailable/failed states without partial values;
5. update the public progress log with the exact artifact result.

Do not widen this milestone into EML generation or unrelated metadata work.

## Following decision point

After complete recipient publication, review the full fixture artifact and select the single largest remaining gap preventing a reconstructable email. Likely candidates include:

- missing or incomplete core message metadata;
- body-form coverage or encoding fidelity;
- attachment table and payload extraction;
- threading/reference fidelity;
- production output wiring from validated diagnostics into structured records.

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

1. EML assembly and exact-preservation policy.
2. Snowflake ingestion.
3. Search and review web application.
4. Semantic search, embeddings, tagging, graph, and LLM/RAG workflows.
5. Distributed orchestration beyond the current local/Docker batch model.

They should begin only after extraction coverage is reliable enough that downstream systems will not institutionalise incomplete data.
