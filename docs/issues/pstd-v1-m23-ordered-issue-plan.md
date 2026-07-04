# PSTD v1 M23 Ordered Issue Plan

## Milestone

M23: Attachment Payload Fidelity.

## Tracking issue

- #138: M23 milestone tracking.

## Ordered implementation issues

| Order | Issue | Title | Purpose |
|---:|---:|---|---|
| 1 | #167 | [M23-I01] Preserve attachment metadata for missing payload rows | Emit metadata-only rows for parsed attachment rows without byte payloads. |
| 2 | #168 | [M23-I02] Add declared size and method fidelity fields | Add declared size, size status, and attachment method fields to attachment records. |
| 3 | #169 | [M23-I03] Update M23 docs and v1 progress reporting | Update docs, v1 progress, and M24 handoff. |

## Execution order

1. Add attachment output fields.
2. Add metadata extraction helpers for size, method, inline status, content ID, filename, and content type.
3. Return unavailable metadata records from table and subnode attachment wiring.
4. Wire metadata-only records into extraction output.
5. Add regression tests for extracted and unavailable rows.
6. Update documentation and output contract.
7. Open a PR and run GitHub Actions CI.
8. Squash merge if CI passes, then close #138 and #167-#169.

## Validation gate

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo run -- --help
cargo run -- batch --help
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
```

## Handoff

M23 should hand off to M24 once CI is green. M24 should focus on batch scale, performance, resume behaviour, progress reporting, and corruption hardening.
