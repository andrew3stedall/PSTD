# PSTD Local and Docker Operator Handoff

## Purpose

This page gives a practical handoff for running PSTD v1 locally or inside Docker.

## Operator assumptions

- You have access to the PST input files you are authorised to process.
- You have a writable output directory with enough space for TAR shards and JSON metadata.
- You are using approved small PST fixtures for validation.
- You do not commit private PST files to the repository.

## Local validation

From the repository root:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo run -- --help
cargo run -- version
cargo run -- inspect --help
cargo run -- batch --help
python -m pstd --help
```

## Docker validation

Build the local image:

```text
docker build -t pstd:local -f docker/Dockerfile .
```

Then run help inside the container:

```text
docker run --rm pstd:local --help
```

## Inspect a PST

Local:

```text
cargo run -- inspect --input <approved-small-fixture.pst>
cargo run -- inspect --input <approved-small-fixture.pst> --json
```

Docker with mounted input/output root:

```text
docker run --rm \
  -v <host-input-dir>:/input:ro \
  -v <host-output-dir>:/output \
  pstd:local inspect --input /input/<file>.pst
```

## Extract one PST

Local:

```text
cargo run -- extract \
  --input <approved-small-fixture.pst> \
  --output <tmp-output>
```

Docker:

```text
docker run --rm \
  -v <host-input-dir>:/input:ro \
  -v <host-output-dir>:/output \
  pstd:local extract \
  --input /input/<file>.pst \
  --output /output/<run-output>
```

Expected output:

```text
<run-output>/
  run_summary.json
  progress.jsonl
  archives/
    <pst_id>_000001.tar
```

## Batch extract PSTs

Local:

```text
cargo run -- batch \
  --input <approved-fixture-directory-or-file> \
  --output <tmp-batch-output>
```

Docker:

```text
docker run --rm \
  -v <host-input-dir>:/input:ro \
  -v <host-output-dir>:/output \
  pstd:local batch \
  --input /input \
  --output /output/batch-run
```

Expected batch output:

```text
<tmp-batch-output>/
  batch_summary.json
  batch_checkpoint.jsonl
  batch_progress.jsonl
  <safe-pst-output-dir>/
    run_summary.json
    progress.jsonl
    archives/
      <pst_id>_000001.tar
```

## Resume behaviour

PSTD batch mode skips existing completed per-PST output directories unless `--overwrite` is set.

Use default resume-by-skip when retrying interrupted work:

```text
cargo run -- batch --input <input-dir> --output <existing-batch-output>
```

Force reprocessing:

```text
cargo run -- batch --input <input-dir> --output <existing-batch-output> --overwrite
```

## Operator review after a run

Review these files first:

1. `batch_summary.json` for aggregate status.
2. `batch_progress.jsonl` for batch start, per-PST transitions, and batch finish events.
3. `batch_checkpoint.jsonl` for one row per processed, skipped, or failed PST.
4. Per-PST `run_summary.json` for individual extraction status.
5. TAR `_pstfast/errors.jsonl` for structured warning and error rows.

## Status interpretation

| Status | Meaning |
|---|---|
| `completed` | The run completed without recorded partial or failed PST items. |
| `completed_with_partial_success` | One or more PSTs completed with recoverable extraction gaps. |
| `completed_with_failures` | Batch continued after one or more PST-level failures. |
| `failed_stopped_early` | Batch stopped after a failure because continue-on-error was disabled. |
| `skipped_completed` | Existing per-PST output was reused because `--overwrite` was not set. |

## Handoff boundary

PSTD v1 is a local/Docker extraction tool. Snowflake ingestion, web UI, search, semantic search, graph, and tagging are post-v1 work.
