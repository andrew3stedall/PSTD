# PSTD v1 Unsupported and Deferred Areas

## Purpose

Record the areas that remain unsupported or deferred after the planned v1 milestone lane. These are explicit boundaries rather than hidden release blockers.

## v1 supported lane

PSTD v1 is a local/Docker Rust + Python extraction tool that emits structured TAR and JSONL outputs for future downstream ingestion and review.

## Deferred post-v1 implementation areas

| Area | Status | Intended next step |
|---|---|---|
| Snowflake ingestion | Post-v1 | Plan schemas, loading, validation, and operational model after v1 handoff. |
| Snowpark Container Services | Post-v1 | Evaluate only after Snowflake ingestion planning is mature. |
| Web UI | Post-v1 | Defer until extracted outputs have a stable storage/query layer. |
| Keyword search | Post-v1 | Build on top of downstream storage. |
| Semantic search and embeddings | Post-v1 | Requires storage, model, and cost decisions. |
| Knowledge graph | Post-v1 | Requires stable entity/reference extraction and downstream graph design. |
| Tagging and review workflow | Post-v1 | Requires UI/storage decisions. |
| Generated EML/download reconstruction | Post-v1 | Use structured v1 outputs as source material if needed later. |
| Distributed orchestration | Post-v1 | Batch mode remains local/Docker and single-process for v1. |

## Parser and extraction caveats

| Area | v1 handling |
|---|---|
| Unsupported PST layouts | Emit explicit unsupported or partial statuses where reachable. |
| ANSI PST support | Remains unsupported unless future evidence requires a focused milestone. |
| Full MAPI property dump mode | Deferred unless fixture-backed evidence proves it is needed. |
| Embedded-message attachment decoding | Metadata is preserved; deep embedded-message payload decoding remains deferred unless bytes are directly available through the current parser path. |
| Exact preservation archive mode | Deferred; v1 focuses on structured extraction and review/search readiness. |
| Broad fixture coverage | Continues as future evidence-driven compatibility work. |

## Non-blocking release-candidate rationale

These areas are non-blocking for v1 because the v1 goal is bounded: produce a local/Docker extractor with documented commands, structured outputs, stable IDs, diagnostics, and recoverable batch behaviour.

Post-v1 work should start with Snowflake ingestion planning, not by expanding the v1 lane.
