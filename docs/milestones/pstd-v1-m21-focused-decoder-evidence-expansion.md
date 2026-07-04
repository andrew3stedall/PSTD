# PSTD v1 M21: Focused Decoder Evidence Expansion

## Goal

Expand decoder evidence after M20 by ensuring the M20 UTF-16 compact attachment-table decoder is visible in compatibility triage outputs and regression coverage.

## Selected evidence gap

| Field | Value |
|---|---|
| Evidence source | M20 `CATW` decoder status output |
| Status family | `utf16_compact_attachment_table_*` |
| Previous behaviour | Only `compact_attachment_table_*` statuses were counted as fixture-backed decoder evidence. |
| M21 behaviour | `CATB` and `CATW` compact decoder statuses are both counted as supported fixture-backed decoder evidence. |
| Fallback requirement | Unsupported, parse-error, and missing-payload cases must keep their existing fallback statuses. |

## Scope

M21 does not add a broad parser path. It adds focused compatibility evidence handling so the review and candidate-selection workflow can see that M20's `CATW` decoder is supported evidence rather than another unknown layout.

## Deliverables

1. Compatibility triage classification for `utf16_compact_attachment_table_*` statuses.
2. Separate supported case output for UTF-16 compact attachment-table evidence.
3. Regression tests for UTF-16 compact decoder evidence.
4. Regression tests for combined `CATB` and `CATW` fixture-backed evidence counts.
5. M21 milestone, implementation, issue-plan, changelog, status, roadmap, and documentation index updates.
6. CI validation before merge.

## Out of scope

- New PST parser layouts.
- Broad attachment parser rewrites.
- Multiple unrelated decoder candidates.
- Snowflake, search, or web UI work.
- Private PST fixture commits.

## Acceptance criteria

- Existing M1-M20 CI remains green.
- `CATB` compact decoder evidence classification still works.
- `CATW` UTF-16 compact decoder evidence is counted as fixture-backed supported evidence.
- Unsupported layout, parse-error, and missing-payload fallback behaviour is unchanged.
- M22 can proceed into body/header fidelity work unless CI reveals a blocking parser issue.

## Validation commands

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo run -- --help
cargo run -- batch --help
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
```
