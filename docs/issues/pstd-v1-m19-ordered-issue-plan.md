# PSTD v1 M19 Ordered Issue Plan

## Branch

`pstd-v1-m19-candidate-selection`

## Issues

| Order | Issue | Title | Notes |
|---:|---:|---|---|
| 1 | #127 | Add focused candidate selection records | Add selection model, ordering, statuses, and tests. |
| 2 | #128 | Export candidate selection JSONL | Write `data/decoder_candidate_selection.jsonl` and manifest/status entries. |
| 3 | #129 | Add M19 docs and workflow guide | Add milestone docs, implementation plan, workflow guide, docs links, status, and changelog. |
| 4 | #130 | Add M19 validation and handoff notes | Finish CI validation and handoff notes. |

## Execution rule

Build on the existing M1-M18 foundation. Keep CI green. Do not add Snowflake, search, web UI, distributed execution, or broad parser rewrites in this slice.

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
