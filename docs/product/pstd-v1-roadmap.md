# PSTD v1 Roadmap

## Roadmap principles

- Build v1 as a local/Docker Rust + Python extraction tool.
- Keep Snowflake, frontend, deployment, search, semantic search, and graph work out of v1 implementation.
- Make the output contract future-ready for Snowflake search, semantic search, tagging, review, download, and graph work.
- Work at milestone and epic level.
- Keep parser expansion narrow, evidence-backed, and test-first.
- Defer phone-only local validation only when necessary, but always record what should be run later.

## Current roadmap position

M1-M24 are implemented through milestone branches and intended for CI validation before merge. M24 hardens batch operation by making discovered, attempted, completed, partial, failed, skipped, and not-run PST counts explicit.

There is **one v1 milestone left after M24**.

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
M21: Focused Decoder Evidence Expansion [implemented, CI validated]
M22: Body and Header Fidelity Expansion [implemented, CI validated]
M23: Attachment Payload Fidelity [implemented, CI validated]
M24: Batch Scale, Performance, and Corruption Hardening [implemented, CI pending]
M25: v1 Release Candidate and Operator Handoff [next, issue #141]
```

## Completed milestone groups

| Range | Outcome |
|---|---|
| M1-M6 | Established the local/Docker extraction archive contract, Rust/Python CLI surface, PST binary primitives, metadata output, recipient/threading foundation, body/attachment output foundation, and batch orchestration. |
| M7-M12 | Added parser depth diagnostics, bounded traversal, payload/subnode traversal, payload wiring, extraction path integration, and attachment table/subnode integration. |
| M13-M24 | Added fixture compatibility coverage, recursive subnode layout exploration, observed-layout triage, fixture-backed decoder expansion, decoder backlog reporting, review workflow outputs, candidate selection outputs, one focused `CATW` attachment-table decoder, UTF-16 compact decoder evidence classification, Unicode HTML body extraction, transport-header metadata, attachment metadata fidelity, and hardened batch progress/status accounting. |

## Completed M24 milestone

### M24: Batch Scale, Performance, and Corruption Hardening

Tracking issue: #139.

M24 improves batch operator diagnostics without adding distributed execution. It adds root-level `batch_progress.jsonl`, preserves discovered-vs-attempted totals, classifies partial-success PSTs, records not-run counts when fail-fast mode stops early, and expands `batch_summary.json` with explicit counters and checkpoint/progress paths.

## Remaining v1 milestone

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
