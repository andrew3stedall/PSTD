# PSTD v1 M6 Ordered Issue Plan

## Branch

`pstd-v1-m6-batch-resume`

## Issues

| Order | Issue | Title | Notes |
|---:|---:|---|---|
| 1 | #60 | Add batch CLI command | Add `pstd batch` without disrupting existing commands. |
| 2 | #61 | Implement PST discovery and per-PST output planning | Discover `.pst` files and allocate safe output directories. |
| 3 | #62 | Add batch checkpoint and summary outputs | Write `batch_checkpoint.jsonl` and `batch_summary.json`. |
| 4 | #63 | Add resume-by-skip behaviour | Skip completed PST outputs unless `--overwrite` is set. |
| 5 | #64 | Add M6 validation and handoff notes | Finish docs and record validation. |

## Execution rule

Build on the existing M1-M5 foundation. Keep CI green. Do not add Snowflake, search, web UI, parallel execution, or distributed orchestration in M6.

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
