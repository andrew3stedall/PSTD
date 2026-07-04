# PSTD v1 M24: Batch Scale, Performance, and Corruption Hardening

## Goal

Prepare the v1 extractor for realistic local and Docker batch runs by making batch status, resume behaviour, progress reporting, and recoverable failure handling clearer and more deterministic.

## Selected hardening gaps

| Gap | M24 change | Operator impact |
|---|---|---|
| Batch total meant attempted rather than discovered | Preserve discovered, attempted, completed, partial, failed, skipped, and not-run counters. | Operators can tell whether a run stopped early or processed the full discovery set. |
| Partial extraction was hidden inside per-PST messages | Classify successful extraction summaries with missing messages, missing attachments, or partial metadata as `partial_success`. | Recoverable extraction gaps are visible in `batch_summary.json`. |
| Root-level batch progress was minimal | Add `batch_progress.jsonl` with batch start, per-PST start/finish, and batch finish events. | Long local/Docker runs can be monitored without opening per-PST outputs. |
| Resume-by-skip lacked previous-run context | Skipped outputs read existing `run_summary.json` where possible. | Operators can see previous run and PST IDs for skipped PSTs. |
| Fail-fast runs did not record not-run count | Batch summaries include `pst_not_run` when `--continue-on-error=false` stops early. | Operators know how much work remains after a failure. |

## Scope

M24 is a batch-orchestration hardening slice. It improves status accounting and diagnostics without changing parser traversal, attachment decoding, Snowflake ingestion, or distributed orchestration.

## Deliverables

1. `batch_summary.json` counters for discovered, attempted, completed, partial, failed, skipped, and not-run PSTs.
2. `batch_progress.jsonl` events for batch and per-PST transitions.
3. Partial-success classification for successful PST runs with recoverable extraction gaps.
4. Resume-by-skip records that preserve previous run IDs and PST IDs when existing `run_summary.json` can be read.
5. CLI batch output that reports all operator counters.
6. Focused regression tests for status derivation and counter aggregation.
7. Documentation updates for the output contract, milestone, implementation plan, issue plan, roadmap, PRD, project status, README, local validation guide, and changelog.

## Out of scope

- Distributed orchestration.
- Snowpark Container Services deployment.
- Snowflake ingestion.
- Search, semantic search, graph work, or web UI.
- Parser or decoder expansion.
- Private fixture commits.

## Acceptance criteria

- Batch status output gives a clear operator view of completed, skipped, failed, partial, and not-run PSTs.
- Resume and continue-on-error behaviour remains deterministic.
- Performance and operational risks are documented with concrete validation commands.
- CI passes before merge.

## Validation commands

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo run -- --help
cargo run -- batch --help
cargo run -- inspect --help
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
```

For approved fixtures only:

```text
cargo run -- batch --input <approved-fixture-directory-or-file> --output <tmp-output>
```
