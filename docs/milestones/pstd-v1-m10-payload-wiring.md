# PSTD v1 M10: Payload Wiring

## Goal

Connect the M8/M9 traversal and payload foundations into concrete wiring helpers for node data blocks, bounded subnode decode planning, and attachment table payload construction.

## Scope

M10 focuses on wiring foundations. It does not add Snowflake, search, web UI, distributed execution, or private fixture data.

## Deliverables

1. Node data-block to property-context loading through BBT/NBT lookups.
2. Structured node payload reports.
3. Bounded subnode decode planning using `ParserLimits::max_subnode_depth`.
4. Attachment table row to property-context conversion.
5. Attachment table row to payload construction.
6. Wiring reports for attachment table payload availability.
7. Synthetic tests for node payload wiring.
8. Synthetic tests for subnode depth-limit planning.
9. Synthetic tests for attachment table payload wiring.
10. Updated docs, issue plan, and project status.

## Out of scope

- Snowflake ingestion.
- Search indexing.
- Web UI.
- Distributed execution.
- Full recursive subnode structure decoding.
- Full real-world PST corpus validation.
- Private PST fixtures.

## Execution order

1. Add node payload wiring helpers.
2. Add bounded subnode decode planning.
3. Add attachment table wiring helpers.
4. Add synthetic tests.
5. Add docs, issues, and handoff notes.
6. Validate through CI.

## Acceptance criteria

- Existing M1-M9 CI remains green.
- An NBT data block can be resolved through BBT and parsed into a property context in a synthetic test.
- Subnode references can produce decode plans with explicit depth-limit status.
- Attachment table rows can be converted into attachment payloads when payload data is present.
- Attachment table rows without payload data are reported explicitly.
- Remaining recursive decoding limitations remain explicit.

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
