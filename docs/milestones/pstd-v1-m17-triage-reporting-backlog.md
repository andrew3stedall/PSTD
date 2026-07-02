# PSTD v1 M17: Compatibility Triage Reporting and Decoder Backlog

## Goal

Turn compatibility triage into an actionable decoder backlog that can guide future parser work without broad speculative rewrites.

## Scope

M17 builds on M15 and M16. It keeps the parser unchanged and adds reporting that converts non-supported compatibility triage cases into prioritized backlog records.

## Deliverables

1. `DecoderBacklogItem` records derived from compatibility triage cases.
2. Priority mapping for unsupported layouts, unparseable attachment tables, and payload mapping gaps.
3. Backlog status mapping for parser work versus payload mapping work.
4. `data/decoder_backlog.jsonl` archive output.
5. Manifest row for decoder backlog output.
6. Run status counter for decoder backlog items.
7. Unit tests proving supported cases are skipped and non-supported cases become backlog items.
8. Documentation, issue plan, changelog, project status updates, and CI validation.

## Out of scope

- Snowflake ingestion.
- Search indexing.
- Web UI.
- Distributed execution.
- New PST decoder expansion.
- Treating unknown layouts as decoded.

## Backlog priority rules

| Category | Priority | Backlog status |
|---|---|---|
| `unsupported_subnode_layout` | High | `decoder_backlog_open` |
| `unparseable_attachment_table` | High | `decoder_backlog_open` |
| `attachment_rows_without_payloads` | Medium | `payload_mapping_backlog_open` |
| Other partial cases | Medium | `payload_mapping_backlog_open` |
| Other non-supported cases | Low | `decoder_backlog_open` |

## Acceptance criteria

- Existing M1-M16 CI remains green.
- Supported compatibility cases do not create backlog rows.
- Non-supported compatibility cases create deterministic backlog rows.
- Extraction archives include `data/decoder_backlog.jsonl`.
- Main status includes `decoder_backlog_items`.
- Docs describe the backlog categories and next workflow.

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
