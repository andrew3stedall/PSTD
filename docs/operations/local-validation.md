# Local Validation

## Purpose

This page lists the checks to run before treating PSTD as release-ready.

## Status

M1, M2, and M3 are implemented on `main`, but local validation is still deferred.

## Commands

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
pstd --help
pstd inspect --help
pstd inspect --input <small-approved-fixture.pst>
pstd inspect --input <small-approved-fixture.pst> --json
pstd extract --input <small-approved-fixture.pst> --output <tmp-output> --manifest-only
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
```

## Fixture guidance

Use synthetic byte fixtures for unit tests where possible. Use small approved PST fixtures for local integration checks.

## Expected metadata output

A metadata-only extraction should create:

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
```

## Failure handling

When validation fails, fix the current baseline before adding a new milestone. Update docs if command behaviour or output shape changes.
