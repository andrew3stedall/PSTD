# PSTD v1 M8 Ordered Issue Plan

## Branch

`pstd-v1-m8-traversal-expansion`

## Issues

| Order | Issue | Title | Notes |
|---:|---:|---|---|
| 1 | #71 | Add bounded BBT traversal | Internal-to-leaf BBT traversal with bounded page count. |
| 2 | #72 | Add bounded node-index traversal | Internal-to-leaf node-index traversal with bounded page count. |
| 3 | #73 | Add property and table parse reports | Add diagnostics without breaking existing APIs. |
| 4 | #74 | Add M8 validation and handoff notes | Finish docs and validation record. |

## Execution rule

Build on the existing M1-M7 foundation. Keep CI green. Do not add Snowflake, search, web UI, distributed execution, or broad parser rewrites in this slice.

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
