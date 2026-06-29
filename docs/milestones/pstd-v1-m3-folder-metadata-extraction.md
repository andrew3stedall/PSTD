# Milestone M3: Folder and Metadata Extraction

## Status

Planned. Not yet implemented.

## Goal

Build on M2 to parse logical PST structures and produce folder inventory plus initial message metadata records.

M3 should make `pstd extract --manifest-only` produce structured archive output populated from real PST folders and message metadata. It does not extract bodies, attachments, recipients, or full threading yet.

## Included epic

- [Epic E3: Logical PST structures and metadata extraction](../epics/pstd-v1-e3-folder-metadata-extraction.md)

## Included issues

1. M3-I01: Stabilise node and block access for logical PST layers.
2. M3-I02: Implement heap-on-node parsing foundation.
3. M3-I03: Implement BTH parsing foundation.
4. M3-I04: Implement property context parsing.
5. M3-I05: Implement table context parsing.
6. M3-I06: Add selected MAPI property registry and value decoding.
7. M3-I07: Traverse folder hierarchy and emit folder inventory.
8. M3-I08: Extract initial message metadata records.
9. M3-I09: Write folder and message metadata to structured archive outputs.
10. M3-I10: Wire metadata-only extraction into the CLI.
11. M3-I11: Add M3 diagnostics, tests, fixture guidance, and handoff notes.

## Dependency order

```text
I01 -> I02 -> I03 -> I04 -> I05 -> I06 -> I07 -> I08 -> I09 -> I10 -> I11
```

## Completion criteria

M3 is complete when the repository has logical node access, heap-on-node parsing, BTH parsing, property context parsing, table context parsing, selected MAPI property decoding, folder hierarchy traversal, folder inventory records, initial message metadata records, and metadata-only archive output.

## Out of scope

- Email body extraction.
- Attachment extraction.
- Recipient extraction.
- Full threading extraction.
- X.400 address resolution.
- Search indexes.
- Snowflake loading.
- Web UI.
- Production deployment.

## Validation expectations

Run later from Codex/laptop or CI:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
pstd inspect --input <approved-small-fixture.pst>
pstd extract --input <approved-small-fixture.pst> --output <tmp-output> --manifest-only
```

Do not commit private PST files. Use tiny synthetic fixtures or approved small PST fixtures only.

## Next milestone

M4 should add recipients, threading fields, conversation metadata, and address resolution on top of M3 metadata extraction.
