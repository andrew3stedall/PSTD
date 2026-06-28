# Milestone M1: Extraction Foundation and Archive Contract

## Status

Implemented and merged to `main` via PR #18.

Local validation remains deferred. Do not treat M1 as release-verified until the documented validation commands have run from Codex/laptop or CI.

## Milestone goal

Create the first implementation-ready foundation for PSTD v1: a local/Docker Rust + Python extraction tool skeleton with a stable structured TAR/JSONL output contract, CLI contract, module boundaries, and deferred testing expectations.

## Included epic

- [Epic E1: Local extraction foundation and archive contract](../epics/pstd-v1-e1-local-extraction-foundation.md)

## Completed issues

1. #7 / M1-I01: Create Rust project skeleton and CLI shell.
2. #8 / M1-I02: Define output archive contract in code-facing docs.
3. #9 / M1-I05: Implement path sanitisation and deterministic ID helpers.
4. #10 / M1-I03: Implement TAR shard writer abstraction.
5. #11 / M1-I04: Implement JSONL writer abstraction and typed placeholder records.
6. #12 / M1-I06: Add status and progress record models.
7. #13 / M1-I07: Add Python CLI wrapper and Rust binary boundary.
8. #14 / M1-I08: Add Docker local run scaffold.
9. #15 / M1-I09: Add deferred testing scaffold and smoke-test placeholders.
10. #16 / M1-I10: Add milestone checklist and handoff notes.

## Implemented foundation

The repository now has:

- Rust crate skeleton named `pstd`.
- CLI entrypoint with `extract`, `inspect`, and `version` commands.
- Runtime config boundary.
- Placeholder engine runner.
- TAR shard writer abstraction.
- JSONL writer abstraction.
- Typed output record models.
- Stable ID helpers.
- Safe archive path helpers.
- Status and progress records.
- PST parser placeholder module boundaries.
- Python wrapper boundary under `python/src/pstd`.
- Local Docker scaffold.
- Deferred testing plan.
- Updated skill output-contract reference.

## Out of scope retained

M1 intentionally does not include:

- Actual PST binary parsing.
- NDB, BBT, NBT, LTP, property context, or table context implementation.
- Real email body extraction.
- Real attachment extraction.
- Snowflake tables or stages.
- Web UI.
- Bun, Vite, or React.
- Keyword search.
- Semantic search.
- Knowledge graph construction.
- Production deployment.
- Secrets, billing changes, production access, or destructive data behaviour.

## Data impact

M1 implements the first placeholder path for the structured TAR + JSONL contract:

- `messages.jsonl`
- `folders.jsonl`
- `manifest.jsonl`
- `errors.jsonl`
- `summary.json`
- `progress.jsonl`

Future milestones will populate the full contract from real PST parsing.

## Validation expectations

Run later from Codex/laptop or CI:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
pstd --help
pstd extract --help
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
```

## Next milestone

M2 should add the PST binary foundation: memory-mapped reader, PST header parsing, strongly typed PST primitives, and low-level block/index planning.
