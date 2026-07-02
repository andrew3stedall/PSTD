# PSTD v1 M16: Fixture-Backed Decoder Expansion

## Goal

Expand decoding only where there is focused regression coverage, while exporting compatibility triage so future fixture runs can be reviewed without re-reading logs.

## Scope

M16 builds on M15 compatibility triage. It adds a narrow compact attachment-table decoder and exports compatibility triage as machine-readable JSONL output.

## Deliverables

1. Compact attachment-table decoder for a focused `CATB` synthetic fixture layout.
2. Regression tests for successful compact attachment payload extraction.
3. Regression tests for compact attachment rows without payload bytes.
4. Compatibility triage classification for compact decoder hits.
5. Machine-readable `data/compatibility_triage.jsonl` archive output.
6. Manifest row for compatibility triage output.
7. Extraction status counters for fixture-backed decoder hits, triage records, and follow-up cases.
8. Documentation, issue plan, changelog, project status updates, and CI validation.

## Out of scope

- Snowflake ingestion.
- Search indexing.
- Web UI.
- Distributed execution.
- Committing mailbox-derived samples.
- Broad PST parser rewrites.
- Treating unknown layouts as decoded.

## Execution order

1. Add compact attachment-table decoder with synthetic regression tests.
2. Extend compatibility triage to classify compact decoder hits.
3. Export compatibility triage records in extraction archives.
4. Update docs and issue tracking.
5. Validate through CI.

## Acceptance criteria

- Existing M1-M15 CI remains green.
- Compact attachment-table payloads can be extracted from synthetic blocks.
- Compact rows without payloads preserve explicit fallback status.
- Triage reports classify compact decoder hits as fixture-backed supported cases.
- Extraction archives include `data/compatibility_triage.jsonl`.
- Main status includes M16 decoder and triage counters.

## Validation commands

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo run -- --help
cargo run -- batch --help
cargo run -- inspect --help
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
```
