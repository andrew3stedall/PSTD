# PSTD v1 M20: Focused Candidate Implementation

## Goal

Implement one selected high-priority candidate from M19 with a narrow parser change, focused regression coverage, and preserved fallback behaviour.

## Selected candidate

| Field | Value |
|---|---|
| Candidate category | `unparseable_attachment_table` |
| Candidate type | Attachment table parser path |
| Implementation target | UTF-16 compact attachment-table rows |
| New magic | `CATW` |
| Existing fallback to preserve | Parse-error reporting for malformed or unsupported table buffers |

## Scope

M20 adds a focused attachment-table decoder variant for compact rows where filename and content-type fields are stored as UTF-16LE byte strings. This is intentionally small and test-backed.

## Deliverables

1. `CATW` attachment table detection.
2. UTF-16LE filename and content-type decoding for compact attachment rows.
3. Attachment payload construction for decoded rows.
4. Parse-error fallback for malformed UTF-16 row metadata.
5. Regression test for successful UTF-16 compact row decoding.
6. Regression test for malformed UTF-16 row fallback.
7. M20 milestone, implementation, issue-plan, changelog, and status docs.
8. CI validation before merge.

## Out of scope

- Broad PST parser rewrites.
- Multiple candidate implementation.
- Snowflake, search, or web UI work.
- Automatic issue creation from extraction outputs.
- Treating unsupported layouts as decoded.

## Acceptance criteria

- Existing M1-M19 CI remains green.
- `CATB` compact attachment tests still pass.
- `CATW` compact attachment tests pass.
- Malformed `CATW` rows retain explicit parse-error fallback.
- No fallback behaviour is removed for unknown table layouts.

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
