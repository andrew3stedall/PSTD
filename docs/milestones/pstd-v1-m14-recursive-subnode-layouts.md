# PSTD v1 M14: Recursive Subnode Layout Exploration

## Goal

Improve recursive subnode handling by classifying loaded subnode layouts, safely following known child-reference layouts, and preserving explicit fallback statuses for unsupported structures.

## Scope

M14 builds on M12 and M13. It remains bounded, local, and compatibility-focused.

## Deliverables

1. Subnode layout classification for table-compatible, known child-reference, and unsupported layouts.
2. Subnode layout reports with layout counts, child-reference counts, unsupported-layout counts, and per-block statuses.
3. Bounded recursive subnode loading for known child-reference layouts.
4. Depth-limit handling for recursive child loading.
5. Main processing-path switch from root-only subnode loading to recursive bounded subnode loading.
6. Extraction status counters for child references, child decodes, unsupported layouts, and attachment table parse errors.
7. Synthetic tests for layout classification, mixed layout compatibility, recursive child loading, and depth-limit behaviour.
8. Documentation, issue plan, changelog, project status updates, and CI validation.

## Out of scope

- Snowflake ingestion.
- Search indexing.
- Web UI.
- Distributed execution.
- Real mailbox data.
- Broad PST parser rewrites.
- Treating unknown recursive layouts as decoded.

## Execution order

1. Add subnode layout classification structures.
2. Add known child-reference layout parsing for bounded recursive tests.
3. Add recursive child loading with duplicate and depth guards.
4. Wire recursive loading into the main processing path.
5. Update docs and issue tracking.
6. Validate through CI.

## Acceptance criteria

- Existing M1-M13 CI remains green.
- Loaded subnode blocks are classified by layout type.
- Known child-reference layouts can be followed within parser limits.
- Depth limits prevent unbounded recursive traversal.
- Unsupported layouts remain explicit in reports and extraction status.
- Main extraction uses recursive bounded loading without reducing existing attachment-path behaviour.

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
