# PSTD Project Status

## Purpose

Provide a single current-state view of what PSTD can do, what is planned next, and what remains unverified.

## Current implementation state

| Area | Status | Notes |
|---|---|---|
| Rust CLI | Implemented and CI validated | `pstd extract`, `pstd inspect`, `pstd batch`, and `pstd version` exist. |
| Structured output contract | M11 integrated and CI validated | M11 writes payload files into TAR archives when helper logic produces payloads. |
| PST byte reader | Implemented foundation and CI validated | Bounded range reads from large PST files. |
| PST header parser | Implemented foundation and CI validated | Validates basic PST magic and version/variant summary. |
| BBT/NBT parsing | Traversal expansion and CI validated | Bounded internal-to-leaf traversal, child-page counts, traversal-error counts, and repeated-offset guards exist. |
| Metadata processing | M11 integrated and CI validated | M11 attempts node payload-to-property context loading for node-index candidates. |
| Recipients/threading | Implemented foundation and CI validated | Recipient/reference outputs, selected MAPI fields, threading helpers, and recipient row conversion exist. |
| Bodies/attachments | M11 integrated and CI validated | M11 emits body payload records, unavailable body rows, and payload archive files where available. |
| Batch orchestration | Implemented foundation and CI validated | Batch discovery, per-PST outputs, checkpoints, summaries, and resume-by-skip behaviour exist. |
| Table/property parse reports | Implemented foundation and CI validated | Table and property parse diagnostics preserve existing APIs. |
| Parser limits | Implemented foundation and CI validated | Explicit parser limits exist for traversal pages, block payload size, and subnode depth. |
| Subnode references | Decode planning and CI validated | Bounded subnode decode planning exists with depth-limit status. |
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

## Latest validation

GitHub Actions validation has passed for the M1-M11 implementation set, including:

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

- Main processing-path integration for attachment table traversal.
- Recursive subnode binary decoding.
- Table-derived attachment payload support from PST nodes.
- Deeper synthetic and public fixture coverage.

M12 should not add Snowflake, search, or web UI work.

## Validation risk

The M1-M11 foundation has CI coverage at the unit, smoke, Docker, and fixture level. Parser-depth risk remains around recursive subnode decoding and attachment table traversal.

Before high-risk parser expansion, continue running the commands in [Validation Guide](../operations/validation-guide.md) and preserve fixture privacy guidance.
