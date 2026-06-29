# Milestone M3: Folder and Metadata Extraction

## Status

Implemented on branch `pstd-v1-m3-folder-metadata`. Pull request review and local validation are pending.

## Goal

Build on M2 to parse logical PST structures and produce folder inventory plus initial message metadata records.

M3 makes `pstd extract --manifest-only` write metadata-only structured archive output using the M3 metadata layer. It does not extract bodies, attachments, recipients, or full threading yet.

## Included epic

- [Epic E3: Logical PST structures and metadata extraction](../epics/pstd-v1-e3-folder-metadata-extraction.md)

## Included issues

1. #32 / M3-I01: Stabilise node and block access for logical PST layers.
2. #33 / M3-I02: Implement heap-on-node parsing foundation.
3. #34 / M3-I03: Implement BTH parsing foundation.
4. #35 / M3-I04: Implement property context parsing.
5. #36 / M3-I05: Implement table context parsing.
6. #37 / M3-I06: Add selected MAPI property registry and value decoding.
7. #38 / M3-I07: Traverse folder hierarchy and emit folder inventory.
8. #39 / M3-I08: Extract initial message metadata records.
9. #40 / M3-I09: Write folder and message metadata to structured archive outputs.
10. #41 / M3-I10: Wire metadata-only extraction into the CLI.
11. #42 / M3-I11: Add M3 diagnostics, tests, fixture guidance, and handoff notes.

## Implemented foundation

- Logical node/block access boundary.
- Heap-on-node parser foundation.
- BTH parser foundation.
- Property context parser foundation.
- Table context parser foundation.
- Selected MAPI property registry and value decoding.
- Root folder inventory emission.
- Initial message metadata/status rows.
- Metadata-only archive output for `folders.jsonl`, `messages.jsonl`, folder inventory, manifest, status records, and summary.
- `pstd extract --manifest-only` metadata path through the existing CLI.
- M3 smoke tests for heap and BTH parsing.

## Out of scope retained

Email bodies, attachments, recipients, full threading, X.400 address resolution, search indexes, Snowflake loading, and web UI remain later work.

## Validation expectations

Run later from Codex/laptop or CI:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
pstd inspect --input <approved-small-fixture.pst>
pstd extract --input <approved-small-fixture.pst> --output <tmp-output> --manifest-only
```

Use only approved small fixtures or synthetic fixtures.

## Next milestone

M4 should add recipients, threading fields, conversation metadata, and address resolution on top of M3 metadata extraction.
