# PSTD v1 Release-Candidate Checklist

## Purpose

Use this checklist before treating PSTD v1 as ready for local or Docker operator use.

## Required CI gate

The M25 PR must pass GitHub Actions CI before merge.

Required CI jobs:

- Rust build.
- `cargo test --all`.
- `cargo clippy --all-targets --all-features -- -D warnings`.
- `cargo fmt --check`.
- Python wrapper installation and `python -m pstd --help`.
- Docker build.
- CLI smoke checks:
  - `cargo run -- --help`
  - `cargo run -- batch --help`
  - `cargo run -- inspect --help`
- Fixture inspect/extract smoke checks when an approved fixture exists in the repository.

## Recommended local gate

Run from the repository root:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo run -- --help
cargo run -- version
cargo run -- inspect --help
cargo run -- batch --help
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
```

## Approved fixture gate

Use only approved small PST fixtures. Do not commit private PST files.

```text
cargo run -- inspect --input <approved-small-fixture.pst>
cargo run -- inspect --input <approved-small-fixture.pst> --json
cargo run -- extract --input <approved-small-fixture.pst> --output <tmp-output>
cargo run -- batch --input <approved-fixture-directory-or-file> --output <tmp-batch-output>
```

## Output review checklist

For single-PST extraction, confirm:

- `run_summary.json` exists.
- `progress.jsonl` exists.
- `archives/` contains one or more TAR shards.
- TAR shards include `_pstfast/summary.json`, `_pstfast/manifest.jsonl`, `_pstfast/errors.jsonl`, and structured `data/*.jsonl` files.

For batch extraction, confirm:

- `batch_summary.json` exists.
- `batch_checkpoint.jsonl` exists.
- `batch_progress.jsonl` exists.
- `batch_summary.json` reports discovered, attempted, completed, partial, failed, skipped, and not-run PST counts.
- Skipped or failed PSTs are visible without opening every per-PST output directory.

## Release-candidate acceptance

PSTD v1 can be treated as release-candidate complete when:

- M25 CI passes.
- M25 PR is merged.
- M1-M25 tracking issues are closed as completed.
- Operator handoff docs are linked from the documentation index.
- Unsupported/deferred areas are documented and non-blocking.

## Non-blocking caveats

- PST layout coverage remains evidence-driven and fixture-dependent.
- Some uncommon or unsupported PST layouts may produce partial outputs or explicit unsupported statuses.
- Snowflake ingestion is post-v1 planning work.
- Web UI, search, embeddings, graph, and tagging are post-v1 implementation work.
