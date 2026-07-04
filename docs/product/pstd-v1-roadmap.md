# PSTD v1 Roadmap

## Roadmap principles

- Build v1 as a local/Docker Rust + Python extraction tool.
- Keep Snowflake, frontend, deployment, search, semantic search, and graph work out of v1 implementation.
- Make the output contract future-ready for Snowflake search, semantic search, tagging, review, download, and graph work.
- Work at milestone and epic level.
- Keep parser expansion narrow, evidence-backed, and test-first.
- Defer phone-only local validation only when necessary, but always record what should be run later.

## Current roadmap position

M1-M25 are implemented through milestone branches and intended for CI validation before merge. M25 closes the planned v1 milestone lane with release-candidate checklist, local/Docker operator handoff, and unsupported/deferred area documentation.

There are **no remaining planned v1 milestones after M25**.

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
M24: Batch Scale, Performance, and Corruption Hardening [implemented, CI validated]
M25: v1 Release Candidate and Operator Handoff [implemented, CI pending]
```

## Completed milestone groups

| Range | Outcome |
|---|---|
| M1-M6 | Established the local/Docker extraction archive contract, Rust/Python CLI surface, PST binary primitives, metadata output, recipient/threading foundation, body/attachment output foundation, and batch orchestration. |
| M7-M12 | Added parser depth diagnostics, bounded traversal, payload/subnode traversal, payload wiring, extraction path integration, and attachment table/subnode integration. |
| M13-M24 | Added fixture compatibility coverage, recursive subnode layout exploration, observed-layout triage, fixture-backed decoder expansion, decoder backlog reporting, review workflow outputs, candidate selection outputs, one focused `CATW` attachment-table decoder, UTF-16 compact decoder evidence classification, Unicode HTML body extraction, transport-header metadata, attachment metadata fidelity, and hardened batch progress/status accounting. |
| M25 | Added release-candidate checklist, local/Docker operator handoff, unsupported/deferred area review, and post-v1 planning boundary. |

## Completed M25 milestone

### M25: v1 Release Candidate and Operator Handoff

Tracking issue: #141.

M25 closes the bounded v1 lane. It documents the final validation gate, local and Docker operating model, expected output review process, unsupported/deferred areas, and post-v1 boundary.

## Post-v1 roadmap

```text
Post-v1 Phase 1: Snowflake ingestion planning
Post-v1 Phase 2: Snowflake ingestion implementation
Post-v1 Phase 3: Search and review web application planning
Post-v1 Phase 4: Semantic search, tagging, graph, and LLM/RAG planning
```

These phases remain outside the completed v1 implementation lane.
