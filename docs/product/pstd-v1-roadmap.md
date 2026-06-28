# PSTD v1 Roadmap

## Roadmap principles

- Build v1 as a local/Docker Rust + Python extraction tool.
- Keep Snowflake, frontend, and deployment concerns out of v1 implementation.
- Make the output contract future-ready for Snowflake search, semantic search, tagging, review, download, and graph work.
- Work at milestone and epic level. The milestone issue order defines execution order.
- Do not require a user prompt after every issue.
- Defer local tests when working from the phone/GitHub connector workflow, but record exactly what should be run later.

## Roadmap overview

```text
M1: Extraction Foundation and Archive Contract [implemented, validation deferred]
  -> Rust CLI shell, archive writer, JSONL contract, Python wrapper boundary, Docker scaffold.

M2: PST Binary Foundation [next]
  -> Memory-mapped reader, PST header parser, primitive PST types, BBT/NBT planning and implementation.

M3: Folder and Metadata Extraction
  -> Folder tree, inventory, property context parsing, table context parsing, message metadata records.

M4: Recipients, Threading, and Address Resolution
  -> Recipients, Message-ID, In-Reply-To, References, Conversation fields, X.400/Exchange address handling.

M5: Bodies and Attachments
  -> Text body, HTML body, body status, normal attachments, inline attachments, attached emails.

M6: Batch Orchestration, Checkpointing, and Progress
  -> Python batch wrapper, worker scheduling, PST-level checkpointing, structured progress logs.

M7: Performance, Resilience, and Release Candidate
  -> Benchmarks, corruption tolerance, memory limits, validation, Docker packaging, documentation.
```

## Milestone M1: Extraction Foundation and Archive Contract

### Status

Implemented and merged to `main` via PR #18. Issues #7-#16 are closed as completed.

Local validation remains deferred.

### Delivered

- Rust crate and `pstd` CLI shell.
- `extract`, `inspect`, and `version` command boundary.
- Structured TAR shard writer.
- JSONL writer.
- Stable ID helpers.
- Safe path helpers.
- Typed output record models.
- Status and progress records.
- Placeholder PST module boundaries.
- Python `pstd` wrapper boundary.
- Docker local run scaffold.
- M1 testing and execution docs.

## Milestone M2: PST Binary Foundation

### Goal

Implement low-level PST file access and structure parsing from scratch.

### Includes

- Memory-mapped PST reader.
- PST header parser.
- Strongly typed PST primitives.
- BBT/NBT traversal.
- Raw block loading.

### Out of scope

- Message bodies.
- Attachments.
- Snowflake.
- Web UI.

## Milestone M3: Folder and Metadata Extraction

### Goal

Enumerate folders and extract message metadata records.

## Milestone M4: Recipients, Threading, and Address Resolution

### Goal

Extract relationship-ready email fields needed for search, review, tagging, and graph work.

## Milestone M5: Bodies and Attachments

### Goal

Extract searchable/reviewable content and attachment artefacts.

## Milestone M6: Batch Orchestration, Checkpointing, and Progress

### Goal

Make PSTD useful across normal batches of 20-100 PSTs and future larger job queues.

## Milestone M7: Performance, Resilience, and Release Candidate

### Goal

Move from working extraction to release candidate quality.

## Future roadmap after v1

```text
V2: Snowflake ingestion
V3: Search and review web application
V4: Knowledge graph and LLM support
```
