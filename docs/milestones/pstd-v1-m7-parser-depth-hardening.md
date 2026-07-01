# PSTD v1 M7: Parser Depth Hardening

## Goal

Reduce parser-depth risk by hardening the BBT/NBT page layer before expanding deeper PST traversal. M7 makes parser behaviour more observable, safer on malformed pages, and easier to validate against real-world PST variability.

## Scope

M7 focuses on PST parser hardening. It should not add Snowflake, search, web UI, or distributed orchestration.

## Deliverables

1. BBT page parse diagnostics:
   - Declared entry count.
   - Parsed entry count.
   - Truncated entry count.
   - Trailer-derived page type and page level.
   - Duplicate block entry count at index level.
2. NBT page parse diagnostics:
   - Declared entry count.
   - Parsed entry count.
   - Truncated entry count.
   - Trailer-derived page type and page level.
   - Duplicate node entry count at index level.
3. Richer BBT/NBT status strings surfaced through inspect and extract status.
4. Tests for complete and truncated BBT/NBT page parsing.
5. Documentation of remaining parser-depth limitations and next hardening steps.

## Out of scope

- Snowflake ingestion.
- Search indexing.
- Web UI.
- Parallel/distributed batch execution.
- Full multi-level B-tree traversal in this first M7 slice.
- Full attachment subnode traversal in this first M7 slice.

## Execution order

1. Add M7 docs and issue plan.
2. Add BBT page diagnostics.
3. Add NBT page diagnostics.
4. Add focused parser-diagnostics tests.
5. Update project status and changelog.
6. Validate through CI.

## Acceptance criteria

- Existing M1-M6 CI remains green.
- BBT/NBT parsers expose explicit parse diagnostics.
- Truncated pages are represented explicitly rather than silently hidden.
- Duplicate BBT/NBT entries are counted at index level.
- `pstd inspect` and extraction status strings include richer BBT/NBT status.
- Parser-depth limitations remain explicit.

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
