# PSTD v1 Deferred Testing Plan

## Purpose

Define validation expectations when PSTD work is created from the phone/GitHub connector workflow and local tests cannot be run until Codex is available on the user's laptop.

## Rule

Do not claim tests passed unless they were actually run.

When tests are not run, state this clearly:

```text
Local tests were not run from the phone/GitHub connector workflow. Tests should be run later from the Codex laptop setup or CI before release.
```

## M1 expected validation commands

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
pstd --help
pstd extract --help
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
```

## M1 validation expectations by issue

| Issue | Validation expectation |
|---|---|
| M1-I01 Rust project skeleton and CLI shell | Cargo format, clippy, tests, CLI help |
| M1-I02 Output archive contract | Documentation review |
| M1-I03 TAR shard writer | Unit tests proving readable TAR output and shard naming |
| M1-I04 JSONL writer | Unit tests proving valid newline-delimited JSON |
| M1-I05 Path and ID helpers | Unit tests for path traversal, duplicates, and stable IDs |
| M1-I06 Status and progress models | Unit tests for JSON serialization |
| M1-I07 Python CLI wrapper | Python CLI help and wrapper behaviour |
| M1-I08 Docker scaffold | Docker build and mounted-directory smoke test |
| M1-I09 Deferred testing scaffold | Documentation review |
| M1-I10 Execution checklist | Documentation review |

## High-risk unverified areas

- Rust project compilation.
- CLI option drift.
- TAR readability.
- JSONL parsing.
- Docker build.
- Python package wrapper behaviour.

## Validation status for this implementation branch

Local tests were not run from the phone/GitHub connector workflow.
