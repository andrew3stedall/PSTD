# PSTD v1 M9 Ordered Issue Plan

## Branch

`pstd-v1-m9-payload-subnode-traversal`

## Issues

| Order | Issue | Title | Notes |
|---:|---:|---|---|
| 1 | #76 | Add parser limits configuration | Add explicit parser limits and limit-aware traversal APIs. |
| 2 | #77 | Add payload block loading through BBT | Resolve and load payload blocks through BBT lookups with size caps. |
| 3 | #78 | Build body and attachment payloads from properties | Build payload records and bytes from parsed property contexts. |
| 4 | #79 | Add subnode-reference reporting | Report node-index subnode references before recursive decoding. |
| 5 | #80 | Add M9 validation and handoff notes | Finish docs and validation record. |

## Execution rule

Build on the existing M1-M8 foundation. Keep CI green. Do not add Snowflake, search, web UI, distributed execution, private PST fixtures, or broad parser rewrites in this slice.

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
