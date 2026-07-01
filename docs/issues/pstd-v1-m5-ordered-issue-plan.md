# PSTD v1 M5 Ordered Issue Plan

## Branch

`pstd-v1-m5-bodies-attachments`

## Issues

| Order | Issue | Title | Notes |
|---:|---:|---|---|
| 1 | #54 | Add body and attachment output scaffolds | Emit `bodies.jsonl` and `attachments.jsonl`. |
| 2 | #55 | Add selected body and attachment MAPI properties | Add body and attachment property constants. |
| 3 | #56 | Implement body output helpers | Stable body records, hashes, paths, and tests. |
| 4 | #57 | Implement attachment output helpers | Safe filenames, stable records, hashes, paths, and tests. |
| 5 | #58 | Add M5 validation and handoff notes | Finish docs and record validation. |

## Execution rule

Build on the existing M1-M4 foundation. Keep CI green. Do not start M6 batch orchestration, Snowflake, search, or UI work in M5.

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
