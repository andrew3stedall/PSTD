# PSTD Project Status

## Purpose

Provide a single current-state view of what PSTD can do, what is planned next, and what remains unverified.

## Current implementation state

| Area | Status | Notes |
|---|---|---|
| Rust CLI | Implemented and CI validated | `pstd extract`, `pstd inspect`, `pstd batch`, and `pstd version` exist. |
| Structured output contract | Extraction integration, pending CI | M11 writes payload bytes into TAR archives when extraction helpers produce payloads. |
| PST byte reader | Implemented foundation and CI validated | Bounded range reads from large PST files. |
| PST header parser | Implemented foundation and CI validated | Validates basic PST magic and version/variant summary. |
| BBT/NBT parsing | Traversal expansion and CI validated | M8 adds bounded internal-to-leaf traversal, child-page counts, traversal-error counts, and repeated-offset guards. |
| Metadata extraction | Extraction integration, pending CI | M11 attempts node payload-to-property context loading for node-index candidates. |
| Recipients/threading | Implemented foundation and CI validated | M4 emits recipient/reference outputs, selected MAPI fields, threading helpers, and recipient row conversion. |
| Bodies/attachments | Extraction integration, pending CI | M11 emits body payload records, unavailable body rows, and payload archive bytes where available. |
| Batch orchestration | Implemented foundation and CI validated | M6 adds batch discovery, per-PST outputs, checkpoints, summaries, and resume-by-skip behaviour. |
| Table/property parse reports | Implemented foundation and CI validated | M8 adds table and property parse diagnostics while preserving existing APIs. |
| Parser limits | Implemented foundation and CI validated | M9 adds explicit parser limits for traversal pages, block payload size, and subnode depth. |
| Subnode references | Decode planning and CI validated | M10 adds bounded subnode decode planning with depth-limit status. |
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
| M11: Extraction Path Integration | Pending | Pending CI |

## Latest validation

GitHub Actions validation has passed for the M1-M10 implementation set, including:

- Rust build.
- Rust unit/integration tests with `cargo test --all`.
- Rust linting with `cargo clippy --all-targets --all-features -- -D warnings`.
- Rust formatting with `cargo fmt --check`.
- Python wrapper install and `python -m pstd --help`.
- Docker image build.
- CLI smoke checks, including `pstd batch --help`.
- Fixture discovery, inspect, and metadata extract when a PST fixture is present.

## Next milestone

M12: Attachment Table and Subnode Integration.

M12 should add:

- Main extraction-path integration for real attachment table traversal.
- Recursive subnode binary decoding.
- Table-derived attachment payload extraction from real PST nodes.
- Deeper synthetic and public fixture coverage.

M12 should not add Snowflake, search, or web UI work.

## Validation risk

The M1-M10 foundation has CI coverage at the unit, smoke, Docker, and fixture level. M11 integrates payload helpers into the main extraction/archive path, but parser-depth risk remains: real-world attachment extraction still depends on recursive subnode decoding and real attachment table traversal.

Before high-risk parser expansion, continue running the commands in [Validation Guide](../operations/validation-guide.md) and preserve fixture privacy guidance.
