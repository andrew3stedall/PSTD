# Milestone M1: Extraction Foundation and Archive Contract

## Status

Planning package for review. This milestone is not yet implemented.

## Milestone goal

Create the first implementation-ready foundation for PSTD v1: a local/Docker Rust + Python extraction tool skeleton with a stable structured TAR/JSONL output contract, CLI contract, module boundaries, and deferred testing expectations.

This milestone prepares agents to implement the first body of work without asking the user to prompt for each issue.

## Alignment decision

Aligned. PSTD's repository purpose is a PST email data extractor using Rust to process PST files. This milestone supports that purpose while keeping future Python, React/Vite, Bun, Snowflake, semantic search, and graph work outside the initial implementation scope.

## Included epic

- [Epic E1: Local extraction foundation and archive contract](../epics/pstd-v1-e1-local-extraction-foundation.md)

## Included issues

1. M1-I01: Create Rust project skeleton and CLI shell.
2. M1-I02: Define output archive contract in code-facing docs.
3. M1-I03: Implement TAR shard writer abstraction.
4. M1-I04: Implement JSONL writer abstraction and typed placeholder records.
5. M1-I05: Implement path sanitisation and deterministic ID helpers.
6. M1-I06: Add structured error and progress event models.
7. M1-I07: Add Python CLI wrapper and Rust binary invocation boundary.
8. M1-I08: Add Docker local execution scaffold.
9. M1-I09: Add deferred testing scaffold and smoke-test placeholders.
10. M1-I10: Add milestone execution checklist and handoff notes.

## Dependency order

```text
I01 -> I02 -> I03 -> I04 -> I05 -> I06 -> I07 -> I08 -> I09 -> I10
```

Some work can proceed in parallel after I01 and I02:

```text
I03 TAR writer
I04 JSONL writer
I05 path/ID helpers
I06 error/progress models
```

The Python wrapper should wait until the Rust CLI shell has a stable command shape.

## Completion criteria

The milestone is complete when the repository has:

- A Rust crate skeleton with CLI entrypoint and module boundaries.
- A Python package skeleton with a CLI wrapper that can invoke the Rust binary boundary.
- A documented TAR + JSONL archive contract.
- A documented set of stable record schemas for future Snowflake-ready output.
- A TAR writer abstraction that can create shard files in tests or placeholder flows.
- JSONL writer abstractions for typed records.
- Structured error and progress event models.
- Deterministic path and ID helper definitions.
- Docker scaffold for local execution.
- Deferred testing notes and suggested commands.
- No Snowflake, frontend, secrets, production access, or destructive behaviour.

## Out of scope

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
- Secrets or billing changes.

## Acceptance criteria

- `pstd extract --input <path> --output <path>` or the final chosen CLI equivalent is documented.
- The output contract describes TAR shard layout, JSONL files, body paths, attachment paths, diagnostics, and stable IDs.
- All planned records include join keys suitable for future message, recipient, body, attachment, folder, error, and tag tables.
- The implementation plan keeps Rust extraction separate from Python orchestration.
- The implementation plan keeps PST reading separate from output writing.
- The milestone issue order is clear enough for agents to continue without prompting after each issue.
- Tests not run from the GitHub connector workflow are explicitly documented.

## Data impact

This milestone defines but does not yet produce real extraction data. It establishes the future v1 output contract:

- `messages.jsonl`
- `recipients.jsonl`
- `message_references.jsonl`
- `bodies.jsonl`
- `attachments.jsonl`
- `folders.jsonl`
- `manifest.jsonl`
- `errors.jsonl`
- `folder_inventory.jsonl`
- `selected_mapi_properties.jsonl`
- `summary.json`
- `progress.jsonl`

## Operational impact

No deployment, secrets, billing, production access, or destructive data behaviour is included. Docker support is local-only scaffolding.

## Validation expectations

Local tests may be deferred until Codex is available on the user's laptop. When implementation starts, suggested commands are:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
python -m pstfast --help
pstd --help
```

If these commands are not available yet because the skeleton has not been implemented, the executing agent must document that gap instead of claiming validation passed.

## Open decisions

- Final binary/package name: `pstd` or `pstfast`.
- Whether M1 should create actual GitHub issues or keep the ordered issue plan in docs until implementation begins.
- Whether to support slash-based milestone branch names in the active GitHub/Codex environment. The suggested fallback is `pstd-v1-m1-foundation`.
