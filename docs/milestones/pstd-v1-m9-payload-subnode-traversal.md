# PSTD v1 M9: Payload and Subnode Traversal

## Goal

Move PSTD from traversal observability toward payload-aware extraction by adding bounded payload block loading, property-derived body and attachment payload builders, parser limits, and subnode-reference reporting.

## Scope

M9 focuses on safe payload and subnode foundations. It does not add Snowflake, search, web UI, distributed execution, or private fixture data.

## Deliverables

1. `ParserLimits` configuration for traversal and payload limits.
2. BBT/NBT traversal APIs that accept parser limits while preserving existing default APIs.
3. Payload block loading through BBT block lookups.
4. Body payload construction from parsed `PropertyContext` values.
5. Attachment payload construction from parsed `PropertyContext` values.
6. Subnode-reference reporting from node-index entries.
7. Unit tests for limits, payload block loading, body payload construction, attachment payload construction, and subnode-reference reporting.
8. Updated docs, issue plan, and project status.

## Out of scope

- Snowflake ingestion.
- Search indexing.
- Web UI.
- Distributed execution.
- Full recursive subnode decoding.
- Full folder, recipient, and attachment table extraction from real-world PSTs.
- Private PST fixtures.

## Execution order

1. Add parser limits.
2. Apply parser limits to BBT/NBT traversal.
3. Add payload block loading through BBT lookup.
4. Add body payload builders from property context.
5. Add attachment payload builders from property context.
6. Add subnode-reference reporting.
7. Add docs, issues, and handoff notes.
8. Validate through CI.

## Acceptance criteria

- Existing M1-M8 CI remains green.
- Default BBT/NBT APIs still work.
- BBT/NBT traversal can be called with explicit parser limits.
- Payload blocks can be resolved through a BBT lookup and loaded with a size cap.
- Body payloads can be constructed from parsed property contexts.
- Attachment payloads can be constructed from parsed property contexts.
- Subnode references can be reported from node-index entries.
- Remaining parser-depth limitations remain explicit.

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
