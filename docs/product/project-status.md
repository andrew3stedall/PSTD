# PSTD Project Status

## Purpose

Provide a single current-state view of what PSTD can do, what is planned next, and what remains unverified.

## Current implementation state

| Area | Status | Notes |
|---|---|---|
| Rust CLI | Implemented and CI validated | `pstd extract`, `pstd inspect`, and `pstd version` exist. |
| Structured output contract | Implemented foundation and CI validated | TAR + JSONL writer and record models exist. |
| PST byte reader | Implemented foundation and CI validated | Bounded range reads from large PST files. |
| PST header parser | Implemented foundation and CI validated | Validates basic PST magic and version/variant summary. |
| BBT/NBT parsing | Skeleton implemented and CI validated | Basic page/index skeletons exist; full traversal remains incomplete. |
| Metadata-only extraction | Implemented foundation and CI validated | Root folder inventory and metadata/status rows are emitted. |
| Recipients/threading | Implemented foundation and CI validated | M4 emits recipient/reference outputs, selected MAPI fields, threading helpers, and recipient row conversion. |
| Bodies/attachments | Not yet implemented | Planned for M5. |
| Batch orchestration | Not yet implemented | Planned for M6. |
| Snowflake/web UI/search | Future work | Out of v1 implementation until later roadmap phases. |

## Merged milestones

| Milestone | Merge PR | Validation status |
|---|---:|---|
| M1: Extraction Foundation and Archive Contract | #18 | CI validated |
| M2: PST Binary Foundation | #30 | CI validated |
| M3: Folder and Metadata Extraction | #43 | CI validated |
| M4: Recipients, Threading, and Address Resolution | #52 and #53 | CI validated |

## Latest validation

GitHub Actions validation has passed for the M1-M4 implementation set, including:

- Rust build.
- Rust unit/integration tests with `cargo test --all`.
- Rust linting with `cargo clippy --all-targets --all-features -- -D warnings`.
- Rust formatting with `cargo fmt --check`.
- Python wrapper install and `python -m pstd --help`.
- Docker image build.
- CLI smoke checks.
- Fixture discovery, inspect, and metadata extract when a PST fixture is present.

## Next milestone

M5: Message Bodies and Attachments.

M5 should add:

- Text body extraction.
- HTML body extraction.
- Body metadata records.
- Attachment metadata records.
- Attachment file extraction.
- Safe filename and archive-path handling.

M5 should not add Snowflake, search, or web UI work.

## Validation risk

The M1-M4 foundation has CI coverage at the unit, smoke, Docker, and fixture level. The remaining validation risk is parser depth: BBT/NBT traversal, folder tables, recipient tables, and real-world PST variability still need broader fixture coverage as M5 and later milestones expand the parser.

Before high-risk parser expansion, continue running the commands in [Validation Guide](../operations/validation-guide.md) and preserve fixture privacy guidance.
