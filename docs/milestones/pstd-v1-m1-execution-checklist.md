# PSTD v1 M1 Milestone Execution Checklist

## Milestone

M1: Extraction Foundation and Archive Contract.

## Branch

Preferred implementation branch:

```text
pstd-v1-m1-foundation
```

If slash-based branch names are supported in the execution environment, use:

```text
milestone/pstd-v1-m1-foundation
```

## Before starting implementation

- [ ] Confirm PRD scope is accepted.
- [ ] Confirm binary/package naming decision: `pstd` vs `pstfast`.
- [ ] Confirm whether to create GitHub issues from the ordered issue plan or work directly from docs.
- [ ] Confirm no Snowflake/frontend/deployment work is included in M1.
- [ ] Start from `main`.
- [ ] Create a new milestone implementation branch.
- [ ] Keep unrelated work out of the branch.

## Execution order

- [ ] M1-I01: Create Rust project skeleton and CLI shell.
- [ ] M1-I02: Define output archive contract in code-facing docs.
- [ ] M1-I05: Implement path sanitisation and deterministic ID helpers.
- [ ] M1-I03: Implement TAR shard writer abstraction.
- [ ] M1-I04: Implement JSONL writer abstraction and typed placeholder records.
- [ ] M1-I06: Add structured error and progress event models.
- [ ] M1-I07: Add Python CLI wrapper and Rust binary invocation boundary.
- [ ] M1-I08: Add Docker local execution scaffold.
- [ ] M1-I09: Add deferred testing scaffold and smoke-test placeholders.
- [ ] M1-I10: Add milestone execution checklist and handoff notes.

## Scope guardrails

Do not include:

- [ ] PST binary parsing.
- [ ] Real message extraction.
- [ ] Real body extraction.
- [ ] Real attachment extraction.
- [ ] Snowflake work.
- [ ] React/Vite/Bun frontend work.
- [ ] Production deployment.
- [ ] Secrets, credentials, billing changes, or destructive data behaviour.
- [ ] Direct commits to `main`.
- [ ] PR merge without explicit user instruction.

## Required implementation behaviours

- [ ] Rust owns extraction and archive writing.
- [ ] Python remains orchestration-only.
- [ ] CLI parsing is separate from extraction logic.
- [ ] PST reader boundary is separate from output writer boundary.
- [ ] Output paths are deterministic and safe.
- [ ] Errors are structured, not string-only.
- [ ] Progress events are structured.
- [ ] JSONL records include stable join keys.
- [ ] Attachments are planned as raw TAR entries, not base64 JSON.
- [ ] EML is not the default canonical output.

## Documentation updates required during implementation

- [ ] Update CLI plan if command names/options change.
- [ ] Update output contract if schemas change.
- [ ] Update deferred testing plan with actual commands.
- [ ] Update README or docs index if new user-facing docs are added.
- [ ] Update known limitations if any unsupported scope is discovered.

## Deferred validation commands

Run later from Codex/laptop or CI:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
pstd --help
pstd extract --help
python -m pstfast --help
docker build -t pstd:local -f docker/Dockerfile .
```

If binary/package names differ, update these commands.

## PR checklist

Every milestone PR should include:

- [ ] Purpose.
- [ ] Scope.
- [ ] Out of scope.
- [ ] Issues covered.
- [ ] Files changed.
- [ ] Behaviour changed.
- [ ] Data impact.
- [ ] Tests actually run.
- [ ] Tests deferred.
- [ ] Known risks.
- [ ] Follow-up work.

## Stop conditions

Stop and report if:

- The implementation requires PST parser decisions outside M1.
- The implementation requires secrets or production access.
- The implementation path would exceed M1 scope.
- The output contract becomes incompatible with the accepted PRD.
- Local tests fail and cannot be fixed within the milestone scope.

## Handoff summary for the first implementation agent

Build the foundation only. Do not parse PSTs yet. Create a working Rust/Python/Docker skeleton that can eventually write the structured TAR + JSONL contract. The next milestone will add real PST binary parsing.
