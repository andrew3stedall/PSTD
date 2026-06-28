# PSTD v1 M1 Dependency Map

## Purpose

Order the first milestone so agents can execute at milestone/epic level without waiting for a user prompt after every issue.

## Milestone

M1: Extraction Foundation and Archive Contract.

## Dependency graph

```text
M1-I01 Rust project skeleton and CLI shell
  -> M1-I03 TAR shard writer
  -> M1-I07 Python CLI wrapper
  -> M1-I08 Docker scaffold

M1-I02 Output archive contract
  -> M1-I03 TAR shard writer
  -> M1-I04 JSONL writer
  -> M1-I05 Path sanitisation and ID helpers
  -> M1-I06 Error and progress models

M1-I03 TAR shard writer
  -> M1-I04 JSONL writer integration
  -> M1-I09 Deferred testing scaffold

M1-I04 JSONL writer
  -> M1-I06 Error and progress models
  -> M1-I09 Deferred testing scaffold

M1-I05 Path sanitisation and ID helpers
  -> M1-I03 TAR shard paths
  -> M1-I04 JSONL record keys

M1-I06 Error and progress models
  -> M1-I07 Python progress/log boundary
  -> M1-I09 Deferred testing scaffold

M1-I07 Python CLI wrapper
  -> M1-I08 Docker scaffold

M1-I08 Docker scaffold
  -> M1-I09 Deferred testing scaffold

M1-I09 Deferred testing scaffold
  -> M1-I10 Milestone execution checklist
```

## Recommended execution sequence

1. M1-I01: Rust project skeleton and CLI shell.
2. M1-I02: Output archive contract.
3. M1-I05: Path sanitisation and deterministic ID helpers.
4. M1-I03: TAR shard writer abstraction.
5. M1-I04: JSONL writer abstraction and typed placeholder records.
6. M1-I06: Structured error and progress event models.
7. M1-I07: Python CLI wrapper and Rust binary invocation boundary.
8. M1-I08: Docker local execution scaffold.
9. M1-I09: Deferred testing scaffold and smoke-test placeholders.
10. M1-I10: Milestone execution checklist and handoff notes.

## Independent work

After M1-I01 and M1-I02 are complete, these can be developed with limited overlap:

- M1-I03 TAR writer.
- M1-I04 JSONL writer.
- M1-I05 path and ID helpers.
- M1-I06 error/progress models.

## Serial work

The Python wrapper should wait for the Rust CLI shell shape.

Docker should wait for at least the Rust and Python skeletons.

Deferred testing docs should be updated after the concrete tool commands are known.

## Paused items

Do not start these in M1:

- PST binary parsing.
- Real message extraction.
- Real attachment extraction.
- Snowflake ingestion.
- Frontend implementation.
- Production deployment.
- Secrets or credential work.

## Sequencing notes

- Output contract must be treated as the first stable consumer-facing interface.
- Parser modules in future milestones should target internal extraction structs, not JSON directly.
- Output writers should convert internal extraction records into stable JSONL schemas.
- Progress reporting should consume structured events rather than inspecting parser internals.

## Missing decisions

- Final binary naming: `pstd` or `pstfast`.
- Whether implementation branches should use slash names where supported or fallback to simple branch names because branch creation with slash names may depend on connector/runtime behaviour.
- Whether M1 should create actual GitHub issues before implementation or keep issue definitions in docs until the first implementation branch starts.
