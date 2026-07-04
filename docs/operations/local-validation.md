# Local Validation

## Purpose

This page lists the checks to run before treating PSTD as release-ready.

## Status

M1-M24 are implemented through milestone branches and are expected to pass GitHub Actions CI before merge. Local validation should still be repeated before the v1 release candidate.

## Commands

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo run -- --help
cargo run -- version
cargo run -- inspect --help
cargo run -- batch --help
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
```

## Approved fixture commands

Use only approved small PST fixtures:

```text
cargo run -- inspect --input <small-approved-fixture.pst>
cargo run -- inspect --input <small-approved-fixture.pst> --json
cargo run -- extract --input <small-approved-fixture.pst> --output <tmp-output> --manifest-only
cargo run -- batch --input <approved-fixture-directory-or-file> --output <tmp-batch-output>
```

## Fixture guidance

Use synthetic byte fixtures for unit tests where possible. Use small approved PST fixtures for local integration checks.

Do not commit private PST files.

## Expected single-PST metadata output

A metadata extraction should create:

```text
run_summary.json
progress.jsonl
archives/<pst-id>_000001.tar
```

The TAR should include:

```text
_pstfast/summary.json
_pstfast/manifest.jsonl
_pstfast/errors.jsonl
_pstfast/folder_inventory.jsonl
data/folders.jsonl
data/messages.jsonl
data/attachments.jsonl
```

## Expected batch output

A batch extraction should create:

```text
batch_summary.json
batch_checkpoint.jsonl
batch_progress.jsonl
<safe-pst-output-dir>/run_summary.json
<safe-pst-output-dir>/progress.jsonl
<safe-pst-output-dir>/archives/<pst-id>_000001.tar
```

`batch_summary.json` should report:

```text
pst_discovered
pst_attempted
pst_completed
pst_partial
pst_failed
pst_skipped
pst_not_run
status
operator_message
checkpoint_path
progress_path
```

`batch_progress.jsonl` should contain root-level operator events such as `batch_started`, `pst_started`, `pst_finished`, and `batch_finished`.

## Failure handling

When validation fails, fix the current baseline before adding a new milestone. Update docs if command behaviour or output shape changes.

For batch validation, inspect `batch_checkpoint.jsonl` and `batch_progress.jsonl` before deleting temporary output. These files should make failed, skipped, partial, and not-run PSTs visible without opening every per-PST output directory.
