# PSTD v1 M2 PST Binary Implementation Plan

## Purpose

Define the implementation shape for the PST binary foundation milestone.

## Constraints

- Do not use `libpff`, `pypff`, `readpst`, Outlook COM, or any PST parsing library.
- Do not commit real private PST files.
- Do not implement folder/message/body/attachment extraction in M2.
- Do not introduce Snowflake, frontend, deployment, production access, secrets, or destructive data behaviour.

## Target modules

```text
src/pst/
  reader.rs
  header.rs
  primitives.rs
  binary.rs
  trailer.rs
  bbt.rs
  nbt.rs
  block.rs
  inspect.rs
```

Existing M1 placeholder modules should be replaced or extended without breaking the `pstd` CLI boundary.

## Reader design

The byte reader should support large PSTs by reading bounded ranges from disk.

Required API shape:

```text
open(path) -> reader
reader.file_size() -> u64
reader.read_at(offset, len) -> Vec<u8>
```

Implementation can use `FileExt`/seek-read patterns first. Memory mapping can be introduced later if helpful.

## Header parser design

The header parser should detect:

- PST-like signature.
- Supported vs unsupported variants.
- Unicode vs ANSI where possible.
- Root references needed by BBT/NBT.
- File-level metadata useful for `inspect`.

Unsupported variants should return structured errors, not panics.

## Primitive types

Prefer strong types over raw integers:

```text
NodeId
BlockId
ByteOffset
PageRef
BlockRef
RootPointers
```

These types should be small, copyable where useful, and clear in debug output.

## Binary helpers

Centralise low-level byte handling:

- Little-endian reads.
- Fixed slices.
- Offset-aware errors.
- Length validation.

Parser modules should not use ad hoc unchecked indexing.

## Trailer parsing

Add page/block trailer structures needed by BBT/NBT work. Full CRC validation may be deferred, but parsed values and diagnostics should be available.

## BBT/NBT skeletons

M2 should implement enough structure to support lookups and diagnostics. It does not need to solve every corrupted or rare page variant.

Required behaviours:

- Parse controlled fixtures.
- Return structured unsupported errors for unhandled variants.
- Avoid panics on malformed input.
- Expose APIs M3 can consume.

## `pstd inspect`

Replace the placeholder with a real inspection path:

```text
pstd inspect --input archive.pst
```

Expected output:

- File size.
- Header variant.
- Supported/unsupported status.
- BBT/NBT root summaries when available.
- Structured diagnostics on invalid input.

Optional flag if simple:

```text
--json
```

## Testing approach

Use tiny synthetic byte fixtures for reader/parser units. Do not use real private PSTs.

Validation commands:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
pstd inspect --help
pstd inspect --input <approved-small-fixture.pst>
```

## Handoff

M3 should use M2's APIs to implement logical PST structures: LTP, heaps, property contexts, table contexts, folder traversal, and message metadata extraction.
