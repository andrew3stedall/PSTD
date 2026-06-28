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
M1: Extraction Foundation and Archive Contract
  -> Rust CLI shell, archive writer, JSONL contract, Python wrapper plan, deferred testing plan.

M2: PST Binary Foundation
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

### Goal

Create the first implementation-ready foundation for PSTD v1 without parsing PST internals yet. Establish CLI shape, archive contract, JSONL schemas, Rust/Python module boundaries, and deferred testing expectations.

### Why first

The output contract and module boundaries must be stable before agents implement PST parsing layers. This prevents later work from embedding Snowflake/frontend assumptions inside the parser and prevents agents from writing ad hoc output formats.

### Expected implementation branch

```text
pstd-v1-m1-foundation
```

The repository skill suggests milestone-style branches. If slash-based branch names are available in the execution environment, use:

```text
milestone/pstd-v1-m1-foundation
```

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

### Includes

- Folder tree.
- Folder inventory.
- Property context parsing.
- Table context parsing.
- `messages.jsonl` records.

## Milestone M4: Recipients, Threading, and Address Resolution

### Goal

Extract relationship-ready email fields needed for search, review, tagging, and graph work.

### Includes

- Recipient records.
- Internet Message-ID.
- In-Reply-To.
- References.
- Conversation-Index.
- Conversation-Topic.
- Raw transport headers.
- X.400/Exchange address preservation and best-effort SMTP resolution.

## Milestone M5: Bodies and Attachments

### Goal

Extract searchable/reviewable content and attachment artefacts.

### Includes

- Plain text bodies.
- HTML bodies.
- Body metadata rows.
- Raw attachment files.
- Attachment metadata rows.
- Inline attachment metadata.
- Attached email preservation and best-effort metadata extraction.

## Milestone M6: Batch Orchestration, Checkpointing, and Progress

### Goal

Make PSTD useful across normal batches of 20-100 PSTs and future larger job queues.

### Includes

- Python CLI wrapper.
- Batch scheduler.
- PST-level checkpointing.
- Retry failed PSTs.
- Skip completed PSTs.
- Console progress.
- `progress.jsonl`.

## Milestone M7: Performance, Resilience, and Release Candidate

### Goal

Move from working extraction to release candidate quality.

### Includes

- Benchmark harness.
- Performance profiles: `fast`, `balanced`, `audit`, `debug`.
- Corruption-tolerant extraction hardening.
- Memory usage checks.
- Docker packaging.
- End-to-end integration tests.
- Final docs.

## Future roadmap after v1

```text
V2: Snowflake ingestion
  Load messages, recipients, bodies, attachments metadata, folder inventory, and errors into Snowflake tables/stages.

V3: Search and review web application
  Keyword search, semantic search, message review, generated email downloads, and tagging.

V4: Knowledge graph and LLM support
  Thread graphs, sender-recipient relationships, attachment relationships, entity extraction, graph-backed retrieval, and RAG workflows.
```

## Known roadmap risks

- PST parser correctness is the hardest part of v1.
- Corrupt PST recovery may require iterative refinement against real fixtures.
- X.400/Exchange address resolution may require optional external mapping files because not every raw Exchange address can be converted from the PST string alone.
- Full message-level resume inside a partially processed PST is more complex than PST-level checkpointing and should not be assumed for M1.
