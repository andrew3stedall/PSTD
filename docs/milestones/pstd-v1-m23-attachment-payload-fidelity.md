# PSTD v1 M23: Attachment Payload Fidelity

## Goal

Tighten attachment payload fidelity by preserving more attachment metadata when byte payloads are extracted, unavailable, empty, or deferred.

## Selected fidelity gaps

| Gap | M23 change | Status impact |
|---|---|---|
| Missing-payload attachment rows lose metadata | Emit metadata-only attachment records for parsed rows whose payload bytes are missing or empty. | Keeps filename, content type, inline flag, content ID, ordinal, declared size, method, archive path, and explicit status. |
| Attachment declared size not surfaced | Add `declared_size_bytes` and `size_status` to attachment records. | Distinguishes `size_matched`, `size_mismatch`, declared-size absence, and payload-unavailable size states. |
| Attachment method not surfaced | Add `attachment_method` to attachment records. | Embedded-message attachment rows can be marked explicitly as deferred when no payload bytes are available. |
| Compact table missing payload rows only affect counters | Emit unavailable records for `CATB` and `CATW` rows with empty payloads. | Keeps compact-table metadata rows visible in `data/attachments.jsonl`. |

## Scope

M23 works only with attachment data already reachable through the existing property-context, table-context, `CATB`, and `CATW` paths. It does not add broad parser traversal, EML reconstruction, or embedded message decoding.

## Deliverables

1. Attachment record fields for declared size, size status, and attachment method.
2. Metadata-only attachment records for parsed rows without payload bytes.
3. Embedded-message payload absence status of `embedded_message_payload_deferred` where attachment method indicates an embedded message.
4. Existing extracted payload records preserved with raw bytes, hash, safe path, and status.
5. Regression tests for metadata-only rows, size status, method preservation, `CATB`/`CATW` extracted payloads, and compact missing payload rows.
6. Documentation updates for the output contract, milestone, implementation plan, issue plan, roadmap, PRD, project status, README, and changelog.

## Out of scope

- Full EML reconstruction.
- Decoding embedded messages into child message records.
- Broad parser rewrites from weak evidence.
- Snowflake, search, embeddings, graph, or web UI work.
- Private fixture commits.

## Acceptance criteria

- Attachment payload fidelity gaps are reduced or explicitly tracked with status reasons.
- Existing `CATB` and `CATW` extracted-payload behaviour remains covered.
- Missing-payload rows preserve metadata instead of becoming generic placeholders.
- Embedded message attachment handling is documented as deferred unless bytes are directly available.
- CI passes before merge.

## Validation commands

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo run -- --help
cargo run -- batch --help
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
```
