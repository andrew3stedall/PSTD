# PSTD v1 M1 Ordered Issue Plan

## Execution rule

Work at milestone/epic level. Do not ask the user to prompt after each issue. Follow this order unless a blocking technical problem is discovered.

## Milestone

[M1: Extraction Foundation and Archive Contract](../milestones/pstd-v1-m1-extraction-foundation.md)

## Epic

[E1: Local Extraction Foundation and Archive Contract](../epics/pstd-v1-e1-local-extraction-foundation.md)

---

## M1-I01: Create Rust project skeleton and CLI shell

### Goal

Create the initial Rust project shape and a CLI entrypoint for local extraction workflows.

### Background

The repo currently documents that PSTD should use Rust to process PST files. The first implementation must create the binary boundary before parser work starts.

### In scope

- Cargo project skeleton.
- Binary entrypoint.
- CLI argument parsing.
- `extract`, `inspect`, and `version` command placeholders if practical.
- Config object mapping CLI options to internal settings.
- Help output.

### Out of scope

- Real PST parsing.
- Real TAR archive output.
- Python wrapper.
- Docker.

### System flow

```text
CLI args -> Config -> placeholder extraction boundary -> structured exit code
```

### Acceptance criteria

- Rust project builds in a local environment once tests are run.
- CLI exposes documented commands or a documented subset.
- CLI does not directly implement extraction logic.
- Config is separated from CLI parsing.
- Unknown commands/invalid args return a non-zero exit code.

### Technical notes

Use the existing Rust project structure reference under `.agents/skills/roles/full-stack-developer/references/rust-project-structure.md`.

### Data considerations

No real extraction data yet.

### UX considerations

Help output should explain input/output options and continue-on-error behaviour.

### Infrastructure and operations considerations

No secrets or production access.

### Dependencies

None.

### Test expectations

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all`
- `pstd --help` or final chosen binary equivalent.

### Documentation required

Update CLI implementation notes if the command shape differs from the plan.

### Risk rating

Medium.

### Open questions

Final binary name: `pstd` or `pstfast`.

---

## M1-I02: Define output archive contract in code-facing docs

### Goal

Create a stable implementation contract for TAR shards, JSONL metadata, body files, attachment files, diagnostics, and stable IDs.

### In scope

- TAR layout.
- JSONL file list.
- Required keys.
- ID generation rules.
- Shard naming rules.
- Deterministic path rules.
- Contract version.

### Out of scope

- Implementing all writers.
- Snowflake table DDL.

### Acceptance criteria

- Contract includes `messages.jsonl`, `recipients.jsonl`, `message_references.jsonl`, `bodies.jsonl`, `attachments.jsonl`, `folders.jsonl`, `manifest.jsonl`, `errors.jsonl`, `folder_inventory.jsonl`, `selected_mapi_properties.jsonl`, `summary.json`, and `progress.jsonl`.
- Contract includes stable keys for future joins and tagging.
- Contract states that attachments are raw files, not base64 JSON payloads.
- Contract states that EML is not the default canonical output.

### Dependencies

M1-I01 can happen in parallel, but the contract should be complete before writer implementation is finalised.

### Test expectations

Docs-only validation; no local tests required unless schema examples are validated by tooling.

### Risk rating

Medium.

---

## M1-I03: Implement TAR shard writer abstraction

### Goal

Create the output abstraction for writing TAR shards with deterministic internal paths.

### In scope

- TAR writer module.
- Shard naming.
- Target shard size setting.
- Add bytes/file entries by archive path.
- Finalise shard metadata.

### Out of scope

- Real PST extraction.
- Compression unless trivial and explicitly behind a flag.

### Acceptance criteria

- Can write at least one TAR shard from placeholder data.
- Can rotate shards by configured size threshold if implemented in M1.
- TAR entries are readable by standard tooling.
- Writer does not require holding the entire archive in memory.

### Dependencies

M1-I01, M1-I02.

### Test expectations

- Unit tests for TAR creation.
- Unit tests for shard path naming.

### Risk rating

Medium.

---

## M1-I04: Implement JSONL writer abstraction and typed placeholder records

### Goal

Create append-friendly JSONL writers for output records.

### In scope

- Typed record structs for placeholder versions of v1 schemas.
- JSONL writer module.
- Write JSONL into TAR output or local staging path as implementation decides.
- Ensure every JSONL row is one valid object per line.

### Out of scope

- Full schema completeness from real PST data.

### Acceptance criteria

- Placeholder message, recipient, body, attachment, folder, manifest, error, and progress rows can serialize.
- JSONL output can be parsed line by line.
- Missing values are not guessed.

### Dependencies

M1-I02, M1-I03.

### Test expectations

- Unit tests for serialization.
- Unit tests for newline-delimited output.

### Risk rating

Low.

---

## M1-I05: Implement path sanitisation and deterministic ID helpers

### Goal

Prevent unsafe archive paths and create stable generated IDs for future joins.

### In scope

- Safe filename/path helper.
- Path traversal prevention.
- Duplicate filename handling.
- Stable helper for `pst_id`, `folder_key`, `message_key`, `body_key`, `attachment_key`, and `recipient_key`.

### Out of scope

- Cryptographic identity guarantees beyond documented v1 strategy.

### Acceptance criteria

- Unsafe paths such as `../../file` cannot escape archive structure.
- Windows-invalid path characters are handled.
- Unicode names are preserved where practical.
- Duplicate attachment names become deterministic unique paths.

### Dependencies

M1-I02.

### Test expectations

- Unit tests for path traversal.
- Unit tests for duplicate names.
- Unit tests for stable ID output from fixed inputs.

### Risk rating

Medium.

---

## M1-I06: Add structured error and progress event models

### Goal

Create reusable error and progress structures for extraction, archive writing, and future parser work.

### In scope

- Error severity enum.
- Error scope/stage enum.
- Recoverable flag.
- Structured progress event model.
- JSON serialization.

### Out of scope

- Real parser error integration.

### Acceptance criteria

- Error records include run/source/message context where known.
- Progress events can be written as JSONL.
- Errors distinguish recoverable item failures from fatal PST failures.

### Dependencies

M1-I02, M1-I04.

### Test expectations

- Unit tests for error serialization.
- Unit tests for progress serialization.

### Risk rating

Low.

---

## M1-I07: Add Python CLI wrapper and Rust binary invocation boundary

### Goal

Create the Python orchestration boundary without making Python responsible for PST parsing.

### In scope

- Python package skeleton.
- `python -m pstfast --help`.
- Wrapper command that locates/invokes the Rust binary boundary.
- Placeholder config pass-through.

### Out of scope

- Batch scheduler.
- Checkpointing.
- Snowflake.

### Acceptance criteria

- Python wrapper is clearly orchestration-only.
- Rust remains the extraction owner.
- Wrapper can print useful errors if the Rust binary is not found.

### Dependencies

M1-I01.

### Test expectations

- `python -m pstfast --help`.
- Basic unit/smoke tests if package tooling exists.

### Risk rating

Medium.

---

## M1-I08: Add Docker local execution scaffold

### Goal

Allow local mounted-directory execution in a Docker container.

### In scope

- Dockerfile.
- Documentation for mounting input/output folders.
- No production deployment.

### Out of scope

- Snowpark Container Services.
- Registries, deployment, credentials, or secrets.

### Acceptance criteria

- Docker build command is documented.
- Docker run command is documented.
- Container uses mounted input and output directories.

### Dependencies

M1-I01, M1-I07.

### Test expectations

- `docker build` and sample `docker run` should be run later when local environment is available.

### Risk rating

Low.

---

## M1-I09: Add deferred testing scaffold and smoke-test placeholders

### Goal

Make validation expectations explicit without claiming unrun tests passed.

### In scope

- Test command list.
- Placeholder smoke tests if practical.
- Deferred testing note in docs.

### Out of scope

- Real PST fixtures.
- Full integration suite.

### Acceptance criteria

- Deferred tests are documented.
- High-risk unverified areas are listed.
- No PR claims tests passed unless commands were actually run.

### Dependencies

M1-I01 through M1-I08.

### Test expectations

Documentation-only if local environment is unavailable.

### Risk rating

Low.

---

## M1-I10: Add milestone execution checklist and handoff notes

### Goal

Prepare the next implementation branch for milestone-level execution.

### In scope

- Checklist.
- Issue execution order.
- Commands to run later.
- Open decisions.
- First implementation branch recommendation.

### Out of scope

- Merging planning PR.
- Starting implementation work unless separately requested.

### Acceptance criteria

- Implementation branch name is stated.
- Ordered issue list is referenced.
- Deferred tests are listed.
- Open decisions are explicit.

### Dependencies

All previous issues.

### Test expectations

Docs-only.

### Risk rating

Low.
