# PSTD v1 M1 Milestone Execution Checklist

## Milestone

M1: Extraction Foundation and Archive Contract.

## Branch

```text
pstd-v1-m1-foundation
```

Slash-based branch names are not required.

## Decisions

- Binary/package name: `pstd`.
- GitHub issues created: #7-#16.
- M1 excludes Snowflake, frontend, deployment, secrets, billing, production access, and destructive behaviour.

## Execution order and coverage

- [x] #7 / M1-I01: Rust project skeleton and CLI shell.
- [x] #8 / M1-I02: Output archive contract.
- [x] #9 / M1-I05: Path sanitisation and deterministic ID helpers.
- [x] #10 / M1-I03: TAR shard writer abstraction.
- [x] #11 / M1-I04: JSONL writer abstraction and typed placeholder records.
- [x] #12 / M1-I06: Status and progress record models.
- [x] #13 / M1-I07: Python CLI wrapper and Rust binary boundary.
- [x] #14 / M1-I08: Docker local run scaffold.
- [x] #15 / M1-I09: Deferred testing scaffold and smoke-test placeholders.
- [x] #16 / M1-I10: Checklist and handoff notes.

## Deferred validation commands

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
pstd --help
pstd extract --help
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
```

## Handoff

M1 provides a Rust/Python/Docker foundation and placeholder archive writer path. The next milestone should add PST binary reading and inspection.
