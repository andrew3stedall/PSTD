# PSTD v1 M7 Ordered Issue Plan

## Branch

`pstd-v1-m7-parser-depth-hardening`

## Issues

| Order | Issue | Title | Notes |
|---:|---:|---|---|
| 1 | #66 | Add BBT page parse diagnostics | Add BBT count, truncation, duplicate, page type, and page level diagnostics. |
| 2 | #67 | Add node index page diagnostics | Add node-index count, truncation, duplicate, page type, and page level diagnostics. |
| 3 | #68 | Add parser diagnostics tests | Add synthetic tests for complete and truncated page parses. |
| 4 | #69 | Add M7 validation and handoff notes | Finish docs and record validation. |

## Execution rule

Build on the existing M1-M6 foundation. Keep CI green. Do not add Snowflake, search, web UI, distributed execution, private PST fixtures, or broad parser rewrites in this slice.

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
