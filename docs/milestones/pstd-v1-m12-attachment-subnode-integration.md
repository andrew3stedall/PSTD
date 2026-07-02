# PSTD v1 M12: Attachment Table and Subnode Integration

## Goal

Improve real attachment-path handling by connecting bounded subnode loading to attachment-table parsing and attachment payload generation in the main processing path.

## Scope

M12 builds on M9-M11. It stays local, bounded, and recoverable.

## Deliverables

1. Bounded subnode root-block loading with status reports.
2. Subnode decode reports with decoded block counts, failure counts, decoded byte counts, and depth-limit status.
3. Attachment-table parsing attempts from loaded subnode blocks.
4. Attachment payload generation from parsed subnode attachment tables.
5. Main processing-path integration for attachment-table payload extraction when message metadata indicates attachments and a matching subnode reference exists.
6. Explicit status rows for missing subnode references, unavailable subnode blocks, unparseable attachment tables, and tables without payload data.
7. Per-payload manifest rows and TAR file writing through the existing M11 archive path.
8. Synthetic tests for subnode loading and subnode-table attachment payload extraction.
9. Documentation, issue plan, changelog, and project status updates.

## Out of scope

- Snowflake ingestion.
- Search indexing.
- Web UI.
- Distributed execution.
- Private PST fixtures.
- Full recursive child-subnode discovery beyond the current bounded root-block integration.
- Full real-world PST corpus validation.

## Execution order

1. Add bounded subnode root-block decode support.
2. Add attachment payload extraction from loaded subnode blocks.
3. Wire subnode attachment extraction into `extract_metadata`.
4. Preserve explicit unavailable status paths.
5. Update docs and issue tracking.
6. Validate through CI.

## Acceptance criteria

- Existing M1-M11 CI remains green.
- Messages with attachment metadata and subnode references attempt attachment subnode loading.
- Loaded subnode blocks are passed through attachment-table parsing.
- Parsed attachment table rows can produce attachment payload records and bytes.
- Unavailable attachment paths remain explicit and recoverable.
- Payloads use the existing archive writer and manifest integration.

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
