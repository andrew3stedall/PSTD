# PSTD v1 M12 Ordered Issue Plan

## Branch

`pstd-v1-m12-attachment-subnode-integration`

## Issues

| Order | Issue | Title | Notes |
|---:|---:|---|---|
| 1 | #92 | Add bounded subnode block loading | Load referenced subnode root blocks with parser limits and reports. |
| 2 | #93 | Extract attachment payloads from subnode table blocks | Parse loaded subnode blocks as attachment tables and convert rows to payloads. |
| 3 | #94 | Wire attachment subnode extraction into processing path | Attempt attachment extraction when message metadata has attachments and a matching subnode reference exists. |
| 4 | #95 | Add M12 validation and handoff notes | Finish docs and validation record. |

## Execution rule

Build on the existing M1-M11 foundation. Keep CI green. Do not add Snowflake, search, web UI, distributed execution, private PST fixtures, or broad parser rewrites in this slice.

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
