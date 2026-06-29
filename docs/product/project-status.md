# PSTD Project Status

## Purpose

Provide a single current-state view of what PSTD can do, what is planned next, and what remains unverified.

## Current implementation state

| Area | Status | Notes |
|---|---|---|
| Rust CLI | Implemented | `pstd extract`, `pstd inspect`, and `pstd version` exist. |
| Structured output contract | Implemented foundation | TAR + JSONL writer and record models exist. |
| PST byte reader | Implemented foundation | Bounded range reads from large PST files. |
| PST header parser | Implemented foundation | Validates basic PST magic and version/variant summary. |
| BBT/NBT parsing | Skeleton implemented | Basic page/index skeletons exist; full traversal remains incomplete. |
| Metadata-only extraction | Implemented foundation | Root folder inventory and metadata/status rows are emitted. |
| Recipients/threading | Not yet implemented | Planned for M4. |
| Bodies/attachments | Not yet implemented | Planned for M5. |
| Batch orchestration | Not yet implemented | Planned for M6. |
| Snowflake/web UI/search | Future work | Out of v1 implementation until later roadmap phases. |

## Merged milestones

| Milestone | Merge PR | Validation status |
|---|---:|---|
| M1: Extraction Foundation and Archive Contract | #18 | Deferred |
| M2: PST Binary Foundation | #30 | Deferred |
| M3: Folder and Metadata Extraction | #43 | Deferred |

## Next milestone

M4: Recipients, Threading, and Address Resolution.

M4 should add:

- Recipient records for to/cc/bcc/reply-to.
- Message-ID, In-Reply-To, and References parsing where available.
- Conversation Index and Conversation Topic fields where available.
- Normalized subject handling.
- Raw transport headers where available.
- Address records and X.400 best-effort resolution.

M4 should not add bodies, attachments, Snowflake, or web UI work.

## Validation risk

The implementation has been created through the GitHub connector workflow. Local Rust, Python, Docker, and fixture validation has not yet been run.

Before starting high-risk parser expansion, run the commands in [Validation Guide](../operations/validation-guide.md).
