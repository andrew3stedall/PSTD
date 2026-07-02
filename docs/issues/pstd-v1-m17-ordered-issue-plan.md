# PSTD v1 M17 Ordered Issue Plan

## Branch

`pstd-v1-m17-triage-reporting-backlog`

## Issues

| Order | Issue | Title | Notes |
|---:|---:|---|---|
| 1 | #117 | Add decoder backlog model from compatibility triage | Derive deterministic backlog rows from non-supported triage cases. |
| 2 | #118 | Export decoder backlog JSONL | Write `data/decoder_backlog.jsonl` and manifest/status entries. |
| 3 | #119 | Update M17 docs and project status | Add milestone docs, implementation plan, docs links, status, and changelog. |
| 4 | #120 | Add M17 validation and handoff notes | Finish CI validation and handoff notes. |

## Execution rule

Build on the existing M1-M16 foundation. Keep CI green. Do not add Snowflake, search, web UI, distributed execution, sample mailbox content, or broad parser rewrites in this slice.

## Validation suite

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
