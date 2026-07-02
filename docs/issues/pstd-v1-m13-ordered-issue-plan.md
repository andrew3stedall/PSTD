# PSTD v1 M13 Ordered Issue Plan

## Branch

`pstd-v1-m13-fixtures-compatibility`

## Issues

| Order | Issue | Title | Notes |
|---:|---:|---|---|
| 1 | #97 | Add attachment subnode compatibility diagnostics | Add parse-error offsets, reasons, and table statuses. |
| 2 | #98 | Expand synthetic attachment payload fixture coverage | Cover complete, missing, unparseable, and partial attachment paths. |
| 3 | #99 | Expand synthetic body payload fixture coverage | Cover text, HTML, and RTF body payload paths. |
| 4 | #100 | Add M13 validation and handoff notes | Finish docs and validation record. |

## Execution rule

Build on the existing M1-M12 foundation. Keep CI green. Do not add Snowflake, search, web UI, distributed execution, real mailbox data, or broad parser rewrites in this slice.

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
