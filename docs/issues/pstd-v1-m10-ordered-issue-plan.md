# PSTD v1 M10 Ordered Issue Plan

## Branch

`pstd-v1-m10-payload-wiring`

## Issues

| Order | Issue | Title | Notes |
|---:|---:|---|---|
| 1 | #82 | Add node payload to property-context wiring | Resolve NBT data blocks through BBT and parse BTH property contexts. |
| 2 | #83 | Add bounded subnode decode planning | Convert subnode references into depth-limited decode plans. |
| 3 | #84 | Add attachment table payload wiring | Convert parsed attachment table rows into attachment payloads. |
| 4 | #85 | Add M10 validation and handoff notes | Finish docs and validation record. |

## Execution rule

Build on the existing M1-M9 foundation. Keep CI green. Do not add Snowflake, search, web UI, distributed execution, private PST fixtures, or broad parser rewrites in this slice.

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
