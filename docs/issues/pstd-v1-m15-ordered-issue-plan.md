# PSTD v1 M15 Ordered Issue Plan

## Branch

`pstd-v1-m15-observed-layout-triage`

## Issues

| Order | Issue | Title | Notes |
|---:|---:|---|---|
| 1 | #107 | Add observed layout compatibility triage reports | Add supported, partial, and parser-work triage categories. |
| 2 | #108 | Add public and sanitized fixture triage guidance | Document safe fixture source, privacy, and issue evidence rules. |
| 3 | #109 | Update M15 docs and project status | Add milestone docs, implementation plan, docs links, status, and changelog. |
| 4 | #110 | Add M15 validation and handoff notes | Finish CI validation and handoff notes. |

## Execution rule

Build on the existing M1-M14 foundation. Keep CI green. Do not add Snowflake, search, web UI, distributed execution, mailbox-derived sample files, or broad parser rewrites in this slice.

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
