# PSTD v1 M6: Batch Orchestration and Resume

## Goal

Extend PSTD from single-PST execution to operational batch execution across multiple PST files, with checkpointing, failure isolation, and resume-friendly output behaviour.

## Scope

M6 focuses on CLI batch orchestration and local operational reliability. It should not add Snowflake, search, or web UI work.

## Deliverables

1. `pstd batch` CLI command.
2. Recursive `.pst` discovery from an input directory.
3. Single-file batch support when `--input` points to one `.pst` file.
4. Per-PST output directories under the batch output root.
5. `batch_checkpoint.jsonl` written incrementally after each PST attempt.
6. `batch_summary.json` written at the end of a batch run.
7. Resume-friendly skip behaviour when a PST output already has `run_summary.json` and `--overwrite` is not set.
8. Failure isolation through `--continue-on-error`.
9. Unit tests for PST detection and safe output names.
10. CI validation with existing Rust, Python, Docker, and CLI smoke checks.

## Out of scope

- Snowflake ingestion.
- Search indexing.
- Web UI.
- Multi-process or distributed execution.
- Full parser-depth expansion.
- Persistent SQLite checkpointing beyond JSONL batch checkpoints.

## Execution order

1. Add M6 docs and issue plan.
2. Add batch configuration and summary/checkpoint models.
3. Add batch PST discovery.
4. Add `pstd batch` CLI command.
5. Add per-PST execution and checkpoint writing.
6. Add resume-by-skip behaviour.
7. Update docs and changelog.
8. Validate with CI.

## Acceptance criteria

- Existing M1-M5 CI remains green.
- `pstd batch --help` works.
- Batch execution can discover `.pst` files.
- Batch execution writes `batch_checkpoint.jsonl` and `batch_summary.json`.
- Completed PSTs can be skipped on later runs unless `--overwrite` is set.
- Failures can be isolated when `--continue-on-error` is enabled.

## Validation commands

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
