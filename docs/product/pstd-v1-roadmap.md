# PSTD v1 Roadmap

## Roadmap principles

- Build v1 as a local/Docker Rust + Python extraction tool.
- Keep Snowflake, frontend, deployment, search, semantic search, and graph work out of v1 implementation.
- Make the output contract future-ready for Snowflake search, semantic search, tagging, review, download, and graph work.
- Work at milestone and epic level.
- Keep parser expansion narrow, evidence-backed, and test-first.
- Defer phone-only local validation only when necessary, but always record what should be run later.

## Current roadmap position

M1-M21 are implemented through milestone branches and intended for CI validation before merge. M21 closes the immediate post-M20 decoder evidence gap by classifying `CATW` status output as fixture-backed evidence.

There are **four v1 milestones left after M21**.

## Roadmap overview

```text
M1: Extraction Foundation and Archive Contract [implemented, CI validated]
M2: PST Binary Foundation [implemented, CI validated]
M3: Folder and Metadata Extraction [implemented, CI validated]
M4: Recipients, Threading, and Address Resolution [implemented, CI validated]
M5: Message Bodies and Attachments [implemented, CI validated]
M6: Batch Orchestration and Resume [implemented, CI validated]
M7: Parser Depth Hardening [implemented, CI validated]
M8: Traversal Expansion [implemented, CI validated]
M9: Payload and Subnode Traversal [implemented, CI validated]
M10: Payload Wiring [implemented, CI validated]
M11: Extraction Path Integration [implemented, CI validated]
M12: Attachment Table and Subnode Integration [implemented, CI validated]
M13: Payload Fixture Expansion and Parser Compatibility [implemented, CI validated]
M14: Recursive Subnode Layout Exploration [implemented, CI validated]
M15: Observed Layout Compatibility and Public Fixture Triage [implemented, CI validated]
M16: Fixture-Backed Decoder Expansion [implemented, CI validated]
M17: Compatibility Triage Reporting and Decoder Backlog [implemented, CI validated]
M18: Decoder Backlog Review Workflow [implemented, CI validated]
M19: Focused Decoder Candidate Selection [implemented, CI validated]
M20: Focused Candidate Implementation [implemented, CI validated]
M21: Focused Decoder Evidence Expansion [implemented, CI pending]
M22: Body and Header Fidelity Expansion [next, issue #137]
M23: Attachment Payload Fidelity [planned, issue #138]
M24: Batch Scale, Performance, and Corruption Hardening [planned, issue #139]
M25: v1 Release Candidate and Operator Handoff [planned, issue #141]
```

## Completed milestone groups

| Range | Outcome |
|---|---|
| M1-M6 | Established the local/Docker extraction archive contract, Rust/Python CLI surface, PST binary primitives, metadata output, recipient/threading foundation, body/attachment output foundation, and batch orchestration. |
| M7-M12 | Added parser depth diagnostics, bounded traversal, payload/subnode traversal, payload wiring, extraction path integration, and attachment table/subnode integration. |
| M13-M21 | Added fixture compatibility coverage, recursive subnode layout exploration, observed-layout triage, fixture-backed decoder expansion, decoder backlog reporting, review workflow outputs, candidate selection outputs, one focused `CATW` attachment-table decoder, and UTF-16 compact decoder evidence classification. |

## Completed M21 milestone

### M21: Focused Decoder Evidence Expansion

Tracking issue: #136.

M21 selected a small post-M20 evidence gap rather than a broad parser rewrite. M20 added `CATW` / `utf16_compact_attachment_table_*` statuses, but compatibility triage only counted `compact_attachment_table_*` statuses as fixture-backed decoder evidence. M21 classifies both `CATB` and `CATW` compact decoder status families as supported fixture-backed evidence and keeps fallback statuses unchanged.

## Remaining v1 milestones

### M22: Body and Header Fidelity Expansion

Tracking issue: #137.

Reduce remaining body/header fidelity gaps that are reachable through the current parser path. Preserve text/HTML status reporting, encoding/size/hash metadata, transport headers where available, selected MAPI properties, and structured errors for unsupported or unavailable data.

### M23: Attachment Payload Fidelity

Tracking issue: #138.

Tighten attachment payload extraction and status reporting. Keep `CATB` and `CATW` behaviour covered, improve supported payload layouts where evidence is strong, and explicitly document embedded message attachment support as supported, partial, or deferred.

### M24: Batch Scale, Performance, and Corruption Hardening

Tracking issue: #139.

Harden realistic local/Docker batch operation. Focus on resume-by-skip behaviour, progress logs, summaries, partial-success handling, corrupted input handling, operator diagnostics, and memory/IO risk documentation.

### M25: v1 Release Candidate and Operator Handoff

Tracking issue: #141.

Close the bounded v1 lane. Run or document the full validation gate, update operator docs, confirm remaining unsupported areas are explicit and non-blocking, and mark the first post-v1 phase as Snowflake ingestion planning.

## Future roadmap after v1

```text
V2: Snowflake ingestion
V3: Search and review web application
V4: Knowledge graph and LLM/RAG support
```

These phases remain out of scope for the v1 implementation lane.
