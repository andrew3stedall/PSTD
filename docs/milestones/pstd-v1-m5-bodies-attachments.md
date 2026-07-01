# PSTD v1 M5: Message Bodies and Attachments

## Goal

Extend PSTD from metadata-only extraction into payload-aware extraction by emitting body records, attachment records, and safe archive paths for extracted message content.

## Scope

M5 focuses on body and attachment output structure plus deterministic conversion helpers. It should remain local CLI / Docker focused and should not add Snowflake, search, or UI work.

## Deliverables

1. Selected MAPI properties for text bodies, HTML bodies, RTF-compressed bodies, attachment metadata, attachment data, content IDs, MIME tags, and inline flags.
2. `data/bodies.jsonl` emitted into the archive contract.
3. `data/attachments.jsonl` emitted into the archive contract.
4. Body helper logic for stable body keys, archive paths, byte counts, and SHA-256 hashes.
5. Attachment helper logic for stable attachment keys, safe filenames, extensions, archive paths, byte counts, and SHA-256 hashes.
6. Explicit body and attachment status values when current parser depth cannot yet surface payload rows.
7. Unit tests for body records, safe filenames, attachment records, and inline metadata.
8. Existing fixture inspect/extract CI remains green.

## Out of scope

- Snowflake ingestion.
- Search indexing.
- Web UI.
- Full Exchange directory lookup.
- Broad BBT/NBT traversal refactors beyond what is needed for safe M5 output.

## Execution order

1. Add M5 docs and issue plan.
2. Add selected body and attachment MAPI properties.
3. Add body conversion helpers and tests.
4. Add attachment conversion helpers and tests.
5. Emit `data/bodies.jsonl` and `data/attachments.jsonl` from the runner.
6. Update manifest and summary counts.
7. Validate with CI and document parser-depth limitations.

## Acceptance criteria

- Existing M1-M4 CI remains green.
- `data/bodies.jsonl` is present in TAR output.
- `data/attachments.jsonl` is present in TAR output.
- Body and attachment records use stable keys and safe archive paths.
- Missing body or attachment payloads are represented explicitly rather than silently skipped.
- M6 batch orchestration remains out of scope.

## Validation commands

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo run -- --help
cargo run -- inspect --help
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
```
