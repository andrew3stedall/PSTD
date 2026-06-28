# PSTD v1 M2 Deferred Testing Plan

## Purpose

Define M2 validation expectations when work is prepared through the phone/GitHub connector workflow and local tests cannot be run immediately.

## Rule

Do not claim tests passed unless they were actually run.

## M2 expected validation commands

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
pstd --help
pstd inspect --help
pstd inspect --input <approved-small-fixture.pst>
```

## Fixture policy

- Do not commit real private PST files.
- Use tiny synthetic byte fixtures for unit tests where possible.
- Use approved small PST fixtures only in local environments or secure fixture storage.
- If a fixture cannot be committed, document the expected local path and how to obtain it.

## M2 validation expectations by issue

| Issue | Validation expectation |
|---|---|
| M2-I01 Reader | Unit tests for bounded range reads and out-of-bounds errors |
| M2-I02 Header | Unit tests for valid/invalid header bytes and unsupported variants |
| M2-I03 Primitives | Unit tests for typed identifiers and formatting |
| M2-I04 Binary helpers | Unit tests for little-endian reads and short buffers |
| M2-I05 Trailers | Unit tests for trailer parsing and invalid lengths |
| M2-I06 BBT | Controlled fixture tests for BBT page/entry parsing |
| M2-I07 NBT | Controlled fixture tests for NBT page/entry parsing |
| M2-I08 Blocks | Unit tests for bounded raw block loading |
| M2-I09 Inspect | CLI smoke test against an approved small fixture |
| M2-I10 Docs | Documentation review |

## High-risk unverified areas

- Binary parser correctness.
- Unsupported PST variants.
- Real-world corrupt PST behaviour.
- BBT/NBT traversal edge cases.
- Absence of committed real PST fixtures.

## Validation status for this planning package

Documentation-only planning. Local tests were not run.
