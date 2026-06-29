# PSTD v1 M3 Ordered Issue Plan

## Milestone

[M3: Folder and Metadata Extraction](../milestones/pstd-v1-m3-folder-metadata-extraction.md)

## Epic

[E3: Logical PST Structures and Metadata Extraction](../epics/pstd-v1-e3-folder-metadata-extraction.md)

## Ordered issue list

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

## Dependency order

```text
#32 -> #33 -> #34 -> #35 -> #36 -> #37 -> #38 -> #39 -> #40 -> #41 -> #42
```

## M3 boundary

M3 is metadata-only. Bodies, attachments, recipients, full threading, address resolution, Snowflake loading, and web UI remain later work.

## Required validation later

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
pstd inspect --input <approved-small-fixture.pst>
pstd extract --input <approved-small-fixture.pst> --output <tmp-output> --manifest-only
```

## Fixture policy

Use tiny synthetic byte fixtures in tests, and approved small PST fixtures only outside the repo.
