# PSTD v1 M3 Deferred Testing Plan

## Purpose

Define validation expectations for the M3 folder and metadata extraction milestone.

## Rule

Do not claim tests passed unless they were actually run.

## M3 expected validation commands

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
pstd inspect --input <approved-small-fixture.pst>
pstd extract --input <approved-small-fixture.pst> --output <tmp-output> --manifest-only
```

## Fixture policy

- Use synthetic byte fixtures for unit tests where practical.
- Use approved small PST fixtures only in local environments or secure fixture storage.
- If a fixture cannot be committed, document the expected local path and how to obtain it.

## M3 validation expectations by issue

| Issue | Validation expectation |
|---|---|
| #32 / M3-I01 Node/block access | Unit tests for node lookup boundary and unsupported cases |
| #33 / M3-I02 Heap-on-node | Unit tests for heap structures and short buffers |
| #34 / M3-I03 BTH | Unit tests for BTH header and entry parsing |
| #35 / M3-I04 Property context | Unit tests for selected property context parsing |
| #36 / M3-I05 Table context | Unit tests for table headers and row extraction skeleton |
| #37 / M3-I06 MAPI registry | Unit tests for property tag mapping and value decoding |
| #38 / M3-I07 Folder hierarchy | Controlled fixture tests for folder rows |
| #39 / M3-I08 Message metadata | Controlled fixture tests for message metadata rows |
| #40 / M3-I09 Archive output | Tests that `folders.jsonl`, `messages.jsonl`, and inventory files are written |
| #41 / M3-I10 CLI wiring | CLI smoke test for metadata-only extraction |
| #42 / M3-I11 Docs | Documentation review |

## High-risk unverified areas

- Logical PST structure correctness.
- Property and table context edge cases.
- Sparse or corrupted PST data.
- Fixture availability.
- Metadata field coverage.

## Validation status

M3 implementation was created through the GitHub connector workflow. Local tests were not run.
