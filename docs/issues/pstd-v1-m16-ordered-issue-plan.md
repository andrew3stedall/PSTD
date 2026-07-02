# PSTD v1 M16 Ordered Issue Plan

## Branch

`pstd-v1-m16-fixture-backed-decoders`

## Issues

| Order | Issue | Title | Notes |
|---:|---:|---|---|
| 1 | #112 | Add compact attachment table decoder | Add focused CATB decoder and regression coverage. |
| 2 | #113 | Export compatibility triage JSONL | Write machine-readable compatibility triage output. |
| 3 | #114 | Update M16 docs and project status | Add milestone docs, implementation plan, docs links, status, and changelog. |
| 4 | #115 | Add M16 validation and handoff notes | Finish CI validation and handoff notes. |

## Execution rule

Build on the existing M1-M15 foundation. Keep CI green. Do not add Snowflake, search, web UI, distributed execution, mailbox-derived sample files, or broad parser rewrites in this slice.

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
