# PSTD v1 M24 Implementation Plan

## Implementation intent

M24 hardens batch operation for realistic local and Docker runs. The focus is not faster parsing or distributed execution; it is deterministic status accounting, progress visibility, and clearer operator diagnostics.

## Current foundation

- `pstd batch` discovers PST files from one file or directory.
- Batch output directories are deterministic per PST path.
- Existing completed outputs are skipped unless `--overwrite` is set.
- Per-PST `run_summary.json` files exist.
- Batch checkpoint rows are appended to `batch_checkpoint.jsonl`.

## Implemented M24 slice

1. Batch summary counters:
   - `pst_discovered`
   - `pst_attempted`
   - `pst_completed`
   - `pst_partial`
   - `pst_failed`
   - `pst_skipped`
   - `pst_not_run`
2. Batch item diagnostics:
   - per-item timings
   - message and attachment extraction counters
   - shard and byte counters
   - existing-output context for skipped PSTs
3. Batch progress stream:
   - `batch_started`
   - `pst_started`
   - `pst_finished`
   - `batch_finished`
4. Partial-success classification:
   - successful extraction summaries with missing messages, missing attachments, `metadata_unavailable`, or partial statuses are classified as `partial_success`.
5. Fail-fast clarity:
   - fail-fast runs preserve discovered totals and report `pst_not_run`.
6. CLI summary:
   - `pstd batch` prints all major counters instead of only completed/failed/skipped.

## Status rules

| Condition | Batch status |
|---|---|
| No PST files discovered | `no_pst_files_found` |
| No failures, no partials, no not-run items | `completed` |
| No failures, at least one partial, no not-run items | `completed_with_partial_success` |
| Failures with `--continue-on-error=true` | `completed_with_failures` |
| Failure stops the run early | `failed_stopped_early` |

## Item status rules

| Condition | Item status |
|---|---|
| Extraction succeeds with no recoverable gaps | `completed` |
| Extraction succeeds with recoverable gaps | `partial_success` |
| Existing output is skipped | `skipped_completed` |
| Extraction errors before recoverable output | `failed` |

## Operational assumptions

- Batch remains local/Docker and single-process for v1.
- Memory and IO improvements must remain bounded and measurable; distributed orchestration is deferred.
- Operators should use `batch_progress.jsonl` and `batch_checkpoint.jsonl` to inspect interrupted or long-running jobs.
- Existing per-PST `run_summary.json` files remain the source of truth for resume-by-skip.

## Definition of done

- CI passes.
- Batch summary and progress outputs expose the new counters.
- Tests cover status and counter aggregation.
- Docs mark M25 as the only remaining v1 milestone.
