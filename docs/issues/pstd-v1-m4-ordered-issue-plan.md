# PSTD v1 M4 Ordered Issue Plan

## Branch

`pstd-v1-m4-recipients-threading`

## Issues

| Order | Issue | Title | Notes |
|---:|---:|---|---|
| 1 | #46 | Add recipient and threading output scaffolds | Establish output files and manifest entries. |
| 2 | #47 | Add selected recipient and threading MAPI properties | Add constants and selected property definitions. |
| 3 | #48 | Implement threading helper logic | Add deterministic subject/reference helper functions and tests. |
| 4 | #49 | Wire threading fields into message metadata | Populate message-level M4 fields where source data exists. |
| 5 | #50 | Parse recipient table rows into recipient records | Convert recipient table rows into stable recipient records. |
| 6 | #51 | Add M4 validation, docs, and handoff notes | Finish documentation and validation record. |

## Execution rule

Build on the existing M1-M3 foundation. Keep CI green after each implementation slice. Do not start M5 body or attachment extraction in M4.

## Validation suite

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo run -- --help
cargo run -- inspect --help
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
```
