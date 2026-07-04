# PSTD Project Status

## Purpose

Provide a single current-state view of what PSTD can do, what is planned next, and what remains unverified.

## Current implementation state

| Area | Status | Notes |
|---|---|---|
| Rust CLI | Implemented and CI validated | `pstd extract`, `pstd inspect`, `pstd batch`, and `pstd version` exist. |
| Structured output contract | M24 batch hardening and CI pending | M24 adds batch-level discovered, attempted, completed, partial, failed, skipped, and not-run counters. |
| PST byte reader | Implemented foundation and CI validated | Bounded range reads from large PST files. |
| PST header parser | Implemented foundation and CI validated | Validates basic PST magic and version/variant summary. |
| BBT/NBT parsing | Traversal expansion and CI validated | Bounded internal-to-leaf traversal, child-page counts, traversal-error counts, and repeated-offset guards exist. |
| Metadata processing | M22 body/header fidelity and CI validated | M22 surfaces `PR_TRANSPORT_MESSAGE_HEADERS` on message records when present. |
| Recipients/threading | Implemented foundation and CI validated | Recipient/reference outputs, selected MAPI fields, threading helpers, and recipient row conversion exist. |
| Bodies/headers | M22 body/header fidelity and CI validated | M22 supports Unicode/string HTML body payloads and preserves binary HTML precedence. |
| Attachments | M23 attachment fidelity and CI validated | M23 preserves metadata-only attachment rows, declared size, size status, method, and deferred embedded-message status. |
| Batch orchestration | M24 batch hardening and CI pending | M24 adds root-level `batch_progress.jsonl`, expanded `batch_summary.json`, deterministic resume-by-skip context, partial-success classification, and not-run counts. |
| Table/property parse reports | M23 attachment fidelity and CI validated | M23 keeps parsed attachment rows visible even when payload bytes are unavailable. |
| Parser limits | Implemented foundation and CI validated | Explicit parser limits exist for traversal pages, block payload size, and subnode depth. |
| Subnode references | M15 compatibility triage and CI validated | M15 summarizes observed subnode layout reports into supported, partial, and unsupported categories. |
| Snowflake/web UI/search | Future work | Out of v1 implementation until later roadmap phases. |

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
| M19: Focused Decoder Candidate Selection | #131 | CI validated |
| M20: Focused Candidate Implementation | #135 | CI validated |
| M21: Focused Decoder Evidence Expansion | #160 | CI validated |
| M22: Body and Header Fidelity Expansion | #166 | CI validated |
| M23: Attachment Payload Fidelity | #171 | CI validated |
| M24: Batch Scale, Performance, and Corruption Hardening | pending | CI pending |

## Latest validation

GitHub Actions validation passed for M23 in PR #171. M24 validation is pending on the milestone PR.

Expected M24 validation includes:

- Rust build.
- Rust unit/integration tests with `cargo test --all`.
- Rust linting with `cargo clippy --all-targets --all-features -- -D warnings`.
- Rust formatting with `cargo fmt --check`.
- Python wrapper install and `python -m pstd --help`.
- Docker image build.
- CLI smoke checks, including `pstd batch --help`.
- Fixture discovery, inspect, and metadata extract when a PST fixture is present.

## Remaining v1 milestones

There is **one v1 milestone left after M24**.

| Order | Milestone | Tracking issue | Purpose |
|---:|---|---:|---|
| 1 | M25: v1 Release Candidate and Operator Handoff | #141 | Close v1 with validation, documentation cleanup, and local/Docker operator handoff. |

## Next milestone

M25: v1 Release Candidate and Operator Handoff.

M25 should add:

- Final validation review across Rust, Python wrapper, Docker, CLI smoke checks, and approved fixture checks.
- Operator handoff documentation for local and Docker execution.
- Explicit unsupported/deferred area review.
- Final v1 release-candidate checklist.

M25 should not add Snowflake, search, or web UI work.

## Validation risk

The M1-M24 foundation has CI coverage at the unit, smoke, Docker, and fixture level once M24 CI passes. Extraction quality still depends on broader observed layout coverage and reviewed validation inputs.

Before high-risk parser expansion, continue running the commands in [Local Validation](../operations/local-validation.md) and preserve fixture handling guidance.
