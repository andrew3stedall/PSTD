# PSTD v1 M24 Ordered Issue Plan

## Milestone

M24: Batch Scale, Performance, and Corruption Hardening.

## Tracking issue

- #139: M24 milestone tracking.

## Ordered implementation issues

| Order | Issue | Title | Purpose |
|---:|---:|---|---|
| 1 | #172 | [M24-I01] Add explicit batch progress and operator counters | Add root-level progress events and clear batch counters. |
| 2 | #173 | [M24-I02] Harden resume, partial-success, and failure classification | Preserve discovered totals, not-run counts, skipped context, and partial-success status. |
| 3 | #174 | [M24-I03] Update M24 operations docs and v1 progress reporting | Update docs, validation guidance, v1 progress, and M25 handoff. |

## Execution order

1. Extend batch summary and item summary structures.
2. Add batch progress event records and append them to `batch_progress.jsonl`.
3. Preserve discovered totals separately from attempted PSTs.
4. Classify successful runs with recoverable gaps as `partial_success`.
5. Preserve previous run and PST IDs for skipped existing outputs where possible.
6. Add focused tests for counter aggregation and status derivation.
7. Update documentation and output contract.
8. Open a PR and run GitHub Actions CI.
9. Squash merge if CI passes, then close #139 and #172-#174.

## Validation gate

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

## Handoff

M24 hands off to M25. M25 should focus on release-candidate validation, operator handoff, final documentation cleanup, and explicit post-v1 planning boundaries.
