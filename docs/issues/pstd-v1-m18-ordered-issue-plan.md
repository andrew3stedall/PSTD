# PSTD v1 M18 Ordered Issue Plan

## Branch

`pstd-v1-m18-backlog-review-workflow`

## Issues

| Order | Issue | Title | Notes |
|---:|---:|---|---|
| 1 | #122 | Add decoder backlog review summary records | Add run-level summary and review status records. |
| 2 | #123 | Add decoder issue candidate records | Group backlog rows into reviewable candidates with checklists. |
| 3 | #124 | Export M18 review workflow outputs | Write review summary and issue candidate JSONL files into the archive. |
| 4 | #125 | Update M18 docs and validation notes | Add docs, links, status, changelog, CI notes, and handoff. |

## Execution rule

Build on the existing M1-M17 foundation. Keep CI green. Do not add Snowflake, search, web UI, distributed execution, or broad parser rewrites in this slice.

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
