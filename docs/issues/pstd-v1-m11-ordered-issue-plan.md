# PSTD v1 M11 Ordered Issue Plan

## Branch

`pstd-v1-m11-extraction-integration`

## Issues

| Order | Issue | Title | Notes |
|---:|---:|---|---|
| 1 | #87 | Integrate node payload loading into extraction path | Use M10 node payload wiring inside main metadata extraction. |
| 2 | #88 | Write extracted payload bytes into archive | Carry body/attachment byte buffers into TAR output. |
| 3 | #89 | Emit explicit unavailable payload rows | Add status rows where body or attachment payloads remain unavailable. |
| 4 | #90 | Add M11 validation and handoff notes | Finish docs and validation record. |

## Execution rule

Build on the existing M1-M10 foundation. Keep CI green. Do not add Snowflake, search, web UI, distributed execution, private PST fixtures, or broad parser rewrites in this slice.

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
