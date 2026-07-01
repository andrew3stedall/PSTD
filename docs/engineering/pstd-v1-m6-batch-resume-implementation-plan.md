# PSTD v1 M6 Implementation Plan

## Implementation intent

M6 adds a local batch execution layer on top of the existing single-PST extractor. The goal is operational safety: deterministic discovery, per-PST isolation, checkpoint records, and resume-friendly skipping.

## Current foundation

- `run_extract` handles one PST and writes a per-PST `run_summary.json`.
- The CLI supports `extract`, `inspect`, and `version`.
- M1-M5 archive output is CI validated.

## Implemented M6 slice

1. `BatchConfig` for batch-level settings.
2. `BatchSummary` and `BatchItemSummary` for run-level reporting.
3. Recursive `.pst` file discovery.
4. Safe per-PST output directory names.
5. `batch_checkpoint.jsonl` appended after each PST attempt.
6. `batch_summary.json` written at the end of batch execution.
7. Resume-by-skip behaviour using existing per-PST `run_summary.json`.
8. `pstd batch` CLI command.
9. Unit tests for `.pst` detection and safe output directory naming.

## Operational behaviour

- Each PST runs through the same `run_extract` path as single-PST extraction.
- Each PST gets an isolated output directory under the batch output root.
- A PST is skipped when its output directory already contains `run_summary.json`, unless `--overwrite` is supplied.
- Failed PSTs are captured in `batch_checkpoint.jsonl` and `batch_summary.json`.
- `--continue-on-error` controls whether the batch keeps going after a failed PST.

## Remaining depth risk

M6 improves orchestration, not parser depth. Real-world body, attachment, recipient, and folder extraction still depends on deeper BBT/NBT, property-context, table-context, and subnode traversal work.

## Future enhancements

- SQLite checkpoint backend.
- Parallel execution controls.
- Retry policies.
- Resource limits.
- Structured run manifests for external orchestration.
- Corpus-level metrics.

## Safety and privacy

Batch mode can process large private archives. Do not commit real PST files, checkpoint outputs, extracted content, or batch summaries containing private paths unless explicitly sanitized.

## Definition of done

- M6 branch keeps CI green.
- `pstd batch --help` is available.
- Batch summary and checkpoint files are documented.
- Resume-by-skip behaviour is implemented.
- Follow-up operational hardening is documented.
