# PQ3 Index Entry Decoding

## Goal

Decode PST BBT and NBT page entries from selected root pages so conversion can progress beyond root traversal.

## Context

PQ2 selected in-bounds Unicode root candidates for the public 271 KB fixture, but traversal still reported zero entries because page metadata and internal child references needed correction.

## Scope

- Read B-tree page metadata from the correct page metadata area.
- Decode internal-page child references from BREF offsets.
- Decode leaf-page BBT and NBT entries.
- Add page diagnostics to inspect output.
- Update tests and conversion-focused docs.

## Out of scope

- Folder hierarchy reconstruction.
- Message table decoding.
- Body, attachment, and recipient coverage.
- Snowflake, UI, search, or downstream loading.

## Acceptance criteria

- Supported pages produce decoded entries.
- Inspect output reports page-level diagnostics.
- Existing parser traversal tests use the corrected page layout.
- Docs park downstream loading work and focus on PST conversion coverage.
