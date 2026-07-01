# PSTD v1 M11: Extraction Path Integration

## Goal

Integrate the payload wiring helpers from M9 and M10 into the main extraction/archive path while keeping unsupported parser paths explicit and recoverable.

## Scope

M11 focuses on safe extraction-path integration. It does not add Snowflake, search, web UI, distributed execution, or private fixture data.

## Deliverables

1. Use parser limits in the main BBT/NBT loading path.
2. Attempt node data-block to property-context loading for node-index candidates.
3. Emit message metadata from successfully loaded property contexts.
4. Build body records and body payloads from loaded property contexts when body properties are present.
5. Emit explicit unavailable body rows when node property contexts or body properties are unavailable.
6. Write extracted body payload bytes into TAR archives.
7. Write extracted attachment payload bytes into TAR archives when attachment payloads are present.
8. Add per-payload manifest records for extracted body and attachment payloads.
9. Update extraction summaries to count extracted attachment payloads.
10. Updated docs, issue plan, and project status.

## Out of scope

- Snowflake ingestion.
- Search indexing.
- Web UI.
- Distributed execution.
- Full recursive subnode binary decoding.
- Full real-world PST corpus validation.
- Private PST fixtures.
- Full attachment-table traversal from real PST nodes.

## Execution order

1. Integrate node payload loading into `extract_metadata`.
2. Integrate body payload generation into metadata output.
3. Add explicit unavailable body rows.
4. Add TAR writer support for body/attachment payload bytes.
5. Add per-payload manifest records.
6. Update docs, issues, and handoff notes.
7. Validate through CI.

## Acceptance criteria

- Existing M1-M10 CI remains green.
- Main extraction attempts node payload-to-property context loading for node-index candidates.
- Successful body payload extraction produces both JSONL metadata and TAR body files.
- Unavailable body/property paths are represented explicitly in JSONL/status output.
- Payload manifest entries are emitted when payload bytes are written.
- Remaining attachment-table and recursive subnode limitations remain explicit.

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
