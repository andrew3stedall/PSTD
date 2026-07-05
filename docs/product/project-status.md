# PSTD Project Status

## Purpose

Provide a single current-state view of what PSTD can do, what is planned next, and what remains unverified.

## Current implementation state

| Area | Status | Notes |
|---|---|---|
| Rust CLI | v1 release candidate and CI validated | `pstd extract`, `pstd inspect`, `pstd batch`, and `pstd version` exist. |
| Structured output contract | v1 release candidate and CI validated | Single-PST and batch output contracts are documented for local/Docker operators. |
| PST byte reader | Implemented foundation and CI validated | Bounded range reads from large PST files. |
| PST header parser | PQ2 candidate selection | Validates PST magic and version/variant summary; PQ2 selects safe root candidates for traversal when available. |
| BBT/NBT parsing | PQ3 index entry decoding | PQ3 corrects B-tree page metadata offsets, internal child references, and page diagnostics. |
| Folder hierarchy | PQ4 folder hierarchy discovery | PQ4 classifies decoded normal/search folder NBT entries, emits folder rows, and records folder discovery counters. |
| Message table discovery | PQ5 message table discovery | PQ5 classifies normal/associated message NBT entries separately from folder/table nodes and records message-table evidence counters. |
| Property/body coverage | PQ6 property and body coverage | PQ6 records property-context coverage and body payload/fallback counters for true message candidates. |
| Selected property dictionary | PQ7 selected property dictionary expansion | PQ7 adds safe String8 aliases for existing selected Unicode string MAPI properties. |
| Metadata processing | PQ7 selected property dictionary expansion | String8 aliases do not explain the public-fixture selected-property gap; selected properties remain 0 of 74 parsed properties. |
| Recipients/threading | Implemented foundation and CI validated | Recipient/reference outputs, selected MAPI fields, threading helpers, and recipient row conversion exist. |
| Bodies/headers | M22 body/header fidelity and CI validated | M22 supports Unicode/string HTML body payloads and preserves binary HTML precedence; PQ7 adds String8 body aliases, but the public fixture still exposes no selected body property. |
| Attachments | M23 attachment fidelity and CI validated | M23 preserves metadata-only attachment rows, declared size, size status, method, and deferred embedded-message status. |
| Batch orchestration | M24 batch hardening and CI validated | M24 adds root-level `batch_progress.jsonl`, expanded `batch_summary.json`, deterministic resume-by-skip context, partial-success classification, and not-run counts. |
| Release-candidate handoff | M25 and CI validated | M25 adds RC checklist, local/Docker operator handoff, and unsupported/deferred area docs. |
| Parser limits | Implemented foundation and CI validated | Explicit parser limits exist for traversal pages, block payload size, and subnode depth. |
| Subnode references | M15 compatibility triage and CI validated | M15 summarizes observed subnode layout reports into supported, partial, and unsupported categories. |
| Snowflake/web UI/search | Parked | Not active until PST conversion coverage is reliable. |

## Merged milestones

| Milestone | Merge PR | Validation status |
|---|---:|---|
| M1: Extraction Foundation and Archive Contract | #18 | CI validated |
| M2: PST Binary Foundation | #30 | CI validated |
| M3: Folder and Metadata Extraction | #43 | CI validated |
| M4: Recipients, Threading, and Address Resolution | #52 and #53 | CI validated |
| M5: Message Bodies and Attachments | #59 | CI validated |
| M6: Batch Orchestration and Resume | #65 | CI validated |
| M7: Parser Depth Hardening | #70 | CI validated |
| M8: Traversal Expansion | #75 | CI validated |
| M9: Payload and Subnode Traversal | #81 | CI validated |
| M10: Payload Wiring | #86 | CI validated |
| M11: Extraction Path Integration | #91 | CI validated |
| M12: Attachment Table and Subnode Integration | #96 | CI validated |
| M13: Payload Fixture Expansion and Parser Compatibility | #101 | CI validated |
| M14: Recursive Subnode Layout Exploration | #106 | CI validated |
| M15: Observed Layout Compatibility and Public Fixture Triage | #111 | CI validated |
| M16: Fixture-Backed Decoder Expansion | #116 | CI validated |
| M17: Compatibility Triage Reporting and Decoder Backlog | #121 | CI validated |
| M18: Decoder Backlog Review Workflow | #126 | CI validated |
| M19: Focused Candidate Selection | #131 | CI validated |
| M20: Focused Candidate Implementation | #135 | CI validated |
| M21: Focused Decoder Evidence Expansion | #160 | CI validated |
| M22: Body and Header Fidelity Expansion | #166 | CI validated |
| M23: Attachment Payload Fidelity | #171 | CI validated |
| M24: Batch Scale, Performance, and Corruption Hardening | #176 | CI validated |
| M25: v1 Release Candidate and Operator Handoff | #180 | CI validated |
| PQ1: Root Decode Diagnostics | #188 | CI validated |
| PQ2: Root Decode Candidate Selection | #193 | CI validated |
| PQ3: Index Entry Decoding | #199 | CI validated |
| PQ4: Folder Hierarchy Discovery | #247 | CI validated |
| PQ5: Message Table Discovery | #262 | CI validated |
| PQ6: Property and Body Coverage | #268 | CI validated |
| PQ7: Selected Property Dictionary Expansion | #274 | CI validated before merge |

## Latest validation

GitHub Actions validation passed for PQ7 in PR #274. The `public-pst-progress` artifact from CI #201 shows the checked-in public PST fixture still emits 11 folders, 1 message candidate, and 0 attachments. PQ7 adds String8 selected-property aliases, but the public fixture still reports 0 selected properties, 74 unknown properties, 0 body payload records, and 1 body fallback row.

Expected PQ8 validation includes:

- Rust build.
- Rust unit/integration tests with `cargo test --all`.
- Rust linting with `cargo clippy --all-targets --all-features -- -D warnings`.
- Rust formatting with `cargo fmt --check`.
- Python wrapper install and `python -m pstd --help`.
- Docker image build.
- CLI smoke checks, including `pstd inspect --help`.
- Fixture discovery, inspect, extract, and public PST progress artifact logging.

## Remaining v1 milestones

There are **no remaining planned v1 milestones after M25**.

## Current active blocker

PQ8 lower-level property-context layout/tag interpretation.

The current focus is solely PST conversion coverage. PQ7 rules out ANSI/String8 aliases as the explanation for the public fixture selected-property gap. The next blocker is understanding why the true message candidate's loaded property context yields 74 parsed but semantically unknown tags.

## Active conversion coverage roadmap

1. PQ8: lower-level property-context layout/tag interpretation.
2. PQ9: body, attachment, and recipient coverage after useful properties are selected.
3. PQ10: fixture corpus and ground-truth comparison.

## Parked work

Snowflake ingestion, UI, search, analytics loading, and other downstream work are parked until PST conversion coverage is reliable and measured against public or sanitized fixtures.

## Validation risk

The M1-M25 foundation has CI coverage at the unit, smoke, Docker, and fixture level. Extraction quality still depends on broader observed layout coverage and reviewed validation inputs.

Before high-risk parser expansion, continue running the commands in [Local Validation](../operations/local-validation.md). For current property dictionary decisions, start with [PQ7 Selected Property Dictionary Expansion](../operations/pq7-selected-property-dictionary.md).
