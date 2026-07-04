# PSTD Project Status

## Purpose

Provide a single current-state view of what PSTD can do, what is planned next, and what remains unverified.

## Current implementation state

| Area | Status | Notes |
|---|---|---|
| Rust CLI | Implemented and CI validated | `pstd extract`, `pstd inspect`, `pstd batch`, and `pstd version` exist. |
| Structured output contract | M21 decoder evidence and CI pending | M17-M21 add decoder backlog, review, candidate selection, focused selected-candidate outputs, and fixture-backed decoder evidence classification. |
| PST byte reader | Implemented foundation and CI validated | Bounded range reads from large PST files. |
| PST header parser | Implemented foundation and CI validated | Validates basic PST magic and version/variant summary. |
| BBT/NBT parsing | Traversal expansion and CI validated | Bounded internal-to-leaf traversal, child-page counts, traversal-error counts, and repeated-offset guards exist. |
| Metadata processing | M21 decoder evidence and CI pending | M21 classifies M20 `CATW` status output as supported fixture-backed decoder evidence. |
| Recipients/threading | Implemented foundation and CI validated | Recipient/reference outputs, selected MAPI fields, threading helpers, and recipient row conversion exist. |
| Bodies/headers | Implemented foundation; fidelity expansion remains | M22 is reserved for body/header fidelity gaps and status/error tightening. |
| Attachments | M21 decoder evidence and CI pending; fidelity expansion remains | M20 adds UTF-16 compact attachment-table decoding for `CATW` rows; M21 makes `CATW` status evidence visible in compatibility triage; M23 is reserved for broader attachment payload fidelity. |
| Batch orchestration | Implemented foundation and CI validated; hardening remains | Batch discovery, per-PST outputs, checkpoints, summaries, and resume-by-skip behaviour exist; M24 is reserved for scale/corruption hardening. |
| Table/property parse reports | M21 decoder evidence and CI pending | M21 keeps malformed, unsupported, and missing-payload paths on explicit fallback statuses. |
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
| M21: Focused Decoder Evidence Expansion | pending | CI pending |

## Latest validation

GitHub Actions validation passed for the M20 implementation commit in PR #135. M21 validation is pending on the milestone PR.

Expected M21 validation includes:

- Rust build.
- Rust unit/integration tests with `cargo test --all`.
- Rust linting with `cargo clippy --all-targets --all-features -- -D warnings`.
- Rust formatting with `cargo fmt --check`.
- Python wrapper install and `python -m pstd --help`.
- Docker image build.
- CLI smoke checks, including `pstd batch --help`.
- Fixture discovery, inspect, and metadata extract when a PST fixture is present.

## Remaining v1 milestones

There are **four v1 milestones left after M21**.

| Order | Milestone | Tracking issue | Purpose |
|---:|---|---:|---|
| 1 | M22: Body and Header Fidelity Expansion | #137 | Reduce body/header extraction gaps while preserving deterministic status reporting. |
| 2 | M23: Attachment Payload Fidelity | #138 | Tighten attachment payload extraction and unsupported-layout reporting. |
| 3 | M24: Batch Scale, Performance, and Corruption Hardening | #139 | Harden realistic batch operation, resume behaviour, progress reporting, and recoverable failures. |
| 4 | M25: v1 Release Candidate and Operator Handoff | #141 | Close v1 with validation, documentation cleanup, and local/Docker operator handoff. |

## Next milestone

M22: Body and Header Fidelity Expansion.

M22 should add:

- Review of current text, HTML, and transport-header extraction statuses.
- Narrow fidelity improvements only where the parser can already reach the relevant payloads.
- Encoding, size, hash, and structured-error preservation.
- Focused regression tests around body/header status transitions.

M22 should not add Snowflake, search, or web UI work.

## Validation risk

The M1-M21 foundation has CI coverage at the unit, smoke, Docker, and fixture level once M21 CI passes. Extraction quality still depends on broader observed layout coverage and reviewed validation inputs.

Before high-risk parser expansion, continue running the commands in [Validation Guide](../operations/validation-guide.md) and preserve fixture handling guidance.
