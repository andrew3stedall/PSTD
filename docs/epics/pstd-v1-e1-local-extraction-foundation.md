# Epic E1: Local Extraction Foundation and Archive Contract

## Epic outcome

Create the first local/Docker implementation foundation for PSTD v1 so agents can build from a stable command shape, archive contract, module boundary, and issue order.

## Current scope

This epic covers foundational implementation only:

- Rust crate and CLI shell.
- Structured configuration boundary.
- TAR shard writer abstraction.
- JSONL writer abstraction.
- Stable output schemas and archive layout.
- Structured error and progress event models.
- Path sanitisation and deterministic ID helpers.
- Python CLI wrapper boundary.
- Docker local run scaffold.
- Deferred testing plan and placeholders.

## Future scope

Later epics will implement:

- PST memory mapping and header parsing.
- PST BBT/NBT traversal.
- LTP/property/table context parsing.
- Folder inventory from real PSTs.
- Message metadata extraction.
- Recipient and threading extraction.
- X.400 address resolution.
- Body extraction.
- Attachment extraction.
- Batch scheduling and checkpointing.
- Performance/resilience hardening.
- Future Snowflake ingestion and web review.

## Out of scope

- Parsing real PST binary structures.
- Extracting real messages.
- Extracting real bodies or attachments.
- Snowflake work.
- Frontend work.
- Secrets, billing, deployment, production access, or destructive behaviour.

## System flow

```text
CLI args
  -> resolved config
  -> Rust extraction boundary
  -> archive/output writer boundary
  -> manifest, errors, summaries, progress events
  -> optional Python wrapper orchestration
```

The first epic should make this flow explicit even if the PST extraction boundary initially uses placeholder records or no-op implementations.

## Ordered issue breakdown

See [Ordered Issue Plan](../issues/pstd-v1-m1-ordered-issue-plan.md).

## Epic success criteria

- Agents can implement the full M1 issue list without needing additional prompts after each issue.
- Rust, Python, output, progress, and error responsibilities are clearly separated.
- Future PST parser work has a stable output writer to target.
- Future Snowflake/search/tagging work can rely on stable IDs and structured output files.
- Local testing gaps are documented rather than hidden.

## Role notes

### Product

Keep the MVP focused on fast local/Docker extraction. Future search, tagging, semantic search, web review, generated downloads, and knowledge graph use cases are requirements for output shape, not implementation scope for this epic.

### Business analysis

This epic should be executed in milestone order, not one issue at a time by user prompt. Each issue has explicit dependencies and acceptance criteria.

### UX

The primary UX is CLI/operator experience. The command must clearly report output locations, partial failures, progress, and deferred validation notes.

### Data

The archive contract must support future joins by stable keys:

- `run_id`
- `pst_id`
- `folder_key`
- `message_key`
- `recipient_key`
- `body_key`
- `attachment_key`
- `reference_key`

### Platform

No production deployment. Docker is local execution scaffolding only.

### Developer feasibility

The epic deliberately avoids PST binary parsing so agents can first establish the scaffolding and contract needed for later parser work.

### Reviewer

Review should focus on scope discipline, output contract stability, and whether any implementation work accidentally assumes Snowflake or web UI dependencies.

## Risks

| Risk | Rating | Mitigation |
|---|---:|---|
| Agents implement output before contract stabilises | Medium | M1 starts with docs and schemas |
| Python boundary becomes hot path | Medium | Keep Python orchestration-only |
| Future Snowflake needs are under-captured | Medium | Include stable IDs, JSONL records, raw headers, selected MAPI properties in contract |
| CLI naming drift | Low | Track as open decision |
| Tests are not run from phone workflow | Medium | Use deferred testing plan and PR wording |

## Documentation required

- PRD.
- Roadmap.
- Milestone definition.
- Epic definition.
- Ordered issue plan.
- Dependency map.
- Data/output contract summary.
- CLI and Rust implementation plan.
- Deferred testing plan.
- Milestone execution checklist.
