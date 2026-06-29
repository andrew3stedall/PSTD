# PSTD Developer Guide

## Purpose

This guide gives developers and coding agents the minimum context needed to work safely in PSTD.

## Repository shape

```text
Cargo.toml                 Rust crate and binary definition
src/                       Rust implementation
  cli.rs                   Command-line interface
  config.rs                Runtime configuration
  engine/                  Extraction orchestration
  output/                  TAR/JSONL output writers and records
  progress.rs              Progress event records
  pst/                     PST parser and metadata extraction layers
python/                    Python wrapper boundary
docker/                    Local Docker build scaffold
tests/                     Rust smoke and unit tests
fixtures/                  Fixture policy and placeholder location
docs/                      Product, engineering, architecture, data, operations, and wiki docs
.agents/skills/            Repo-scoped planning and execution skills
```

## Main commands

```text
pstd --help
pstd inspect --help
pstd inspect --input <approved-small-fixture.pst>
pstd inspect --input <approved-small-fixture.pst> --json
pstd extract --input <approved-small-fixture.pst> --output <tmp-output> --manifest-only
```

## Validation commands

Run before claiming a branch is validated:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
pstd --help
pstd inspect --help
pstd inspect --input <approved-small-fixture.pst>
pstd extract --input <approved-small-fixture.pst> --output <tmp-output> --manifest-only
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
```

If these are not run, say so in the PR.

## Fixture policy

- Do not commit private PST files.
- Prefer tiny synthetic byte fixtures for unit tests.
- Use approved small PST fixtures only locally or in secure fixture storage.
- Document fixture assumptions when tests require a local PST.

## Implementation boundaries

### Rust owns

- PST byte reading and parsing.
- Metadata extraction.
- Output writing.
- TAR shard creation.
- JSONL record creation.
- Progress and status records.

### Python owns

- Operator convenience wrapper.
- Future batch orchestration.
- Future reports.

Python must not parse PST internals or sit in the per-message hot path.

### Future systems consume outputs

Snowflake, search, graph, and web UI work should consume the structured TAR + JSONL contract. They should not be introduced into the Rust parser milestones unless a milestone explicitly says so.

## Current parser layers

| Layer | Module | Status |
|---|---|---|
| Byte reader | `src/pst/reader.rs` | Foundation implemented |
| Header parser | `src/pst/header.rs` | Foundation implemented |
| Primitive IDs | `src/pst/primitives.rs` | Foundation implemented |
| Binary helpers | `src/pst/binary.rs` | Foundation implemented |
| BBT/NBT skeletons | `src/pst/bbt.rs`, `src/pst/nbt.rs` | Skeleton implemented |
| Logical node access | `src/pst/logical.rs` | Foundation implemented |
| Heap/BTH | `src/pst/heap.rs`, `src/pst/bth.rs` | Foundation implemented |
| Property/table contexts | `src/pst/property_context.rs`, `src/pst/table_context.rs` | Scaffold implemented |
| MAPI registry | `src/pst/mapi.rs` | Selected properties only |
| Folder/message metadata | `src/pst/folder_tree.rs`, `src/pst/message_metadata.rs` | Metadata/status output foundation |

## Output contract

The canonical v1 output is structured TAR + JSONL. EML is not the default output.

Read:

- [Output Contract Summary](../data/pstd-v1-output-contract-summary.md)
- [Output Contract Reference](../../.agents/skills/roles/data/references/output-contract.md)

## Work process

1. Read `AGENTS.md` and `.agents/skills/README.md`.
2. Identify the milestone or issue scope.
3. Check the relevant milestone, epic, dependency map, and implementation plan.
4. Keep changes inside scope.
5. Add/update tests where practical.
6. Update docs and changelog.
7. Record validation that was run or deferred.

## Pull request checklist

Every PR should include:

- Purpose.
- Scope.
- Files changed.
- Tests run.
- Tests deferred.
- Documentation updated.
- Data impact.
- Operational impact.
- Follow-up work.

## Current limitations

- M1-M3 were merged without local validation.
- Real-world PST parsing is incomplete.
- Folder/message metadata output is still foundational.
- Recipients, threading, bodies, and attachments are not implemented.
