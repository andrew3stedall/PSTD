# PSTD v1 M8: Traversal Expansion

## Goal

Expand PSTD traversal beyond root-page parsing while keeping traversal bounded, observable, and safe.

## Scope

M8 focuses on traversal expansion and parse observability. It does not add Snowflake, search, web UI, or distributed execution.

## Deliverables

1. Bounded BBT traversal from internal pages to leaf pages.
2. Bounded node-index traversal from internal pages to leaf pages.
3. Traversal limits to avoid repeated parsing.
4. Repeated-offset detection for BBT and node-index page traversal.
5. Child-page discovery counts and traversal-error counts.
6. Table-context parse reports for declared and parsed rows and columns.
7. Property-context parse reports for selected, unknown, skipped, and decode-error counts.
8. Synthetic tests for internal-to-leaf traversal.
9. Synthetic tests for table and property parse reports.
10. Updated status and handoff docs.

## Out of scope

- Snowflake ingestion.
- Search indexing.
- Web UI.
- Distributed execution.
- Full attachment subnode traversal.
- Full semantic validation of every PST B-tree variant.

## Execution order

1. Add bounded BBT traversal.
2. Add bounded node-index traversal.
3. Add traversal tests.
4. Add table-context parse reports.
5. Add property-context parse reports.
6. Add docs, issues, and handoff notes.
7. Validate through CI.

## Acceptance criteria

- Existing M1-M7 CI remains green.
- BBT traversal can move from an internal root page to a leaf page in a synthetic test.
- Node-index traversal can move from an internal root page to a leaf page in a synthetic test.
- Traversal status includes parsed pages, discovered child pages, entries, truncated entries, duplicate entries, and traversal errors.
- Table-context parse reports expose declared and parsed row and column counts.
- Property-context parse reports expose selected, unknown, skipped, and decode-error counts.
- Parser limitations remain explicit.

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
