# PSTD v1 M20 Ordered Issue Plan

## Branch

`pstd-v1-m20-focused-candidate`

## Issues

| Order | Issue | Title | Notes |
|---:|---:|---|---|
| 1 | #132 | Add UTF16 compact attachment table decoder | Add `CATW` decoder path for one focused high-priority candidate. |
| 2 | #133 | Add focused decoder tests | Add success and invalid-row regression coverage. |
| 3 | #134 | Update M20 docs and validation notes | Add docs, links, status, changelog, CI notes, and handoff. |

## Execution rule

Implement one focused parser candidate only. Keep CI green. Do not add Snowflake, search, web UI, distributed execution, or broad parser rewrites in this slice.

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
