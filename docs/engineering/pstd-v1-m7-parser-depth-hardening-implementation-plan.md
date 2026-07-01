# PSTD v1 M7 Implementation Plan

## Implementation intent

M7 starts parser-depth hardening at the BBT/NBT page boundary. The goal is not to overclaim full PST traversal, but to make the current parser safer and more observable before adding multi-level traversal and subnode parsing.

## Current foundation

- M2 added BBT/NBT root-page parsing.
- M3-M5 rely on BBT/NBT status and candidate entries.
- M6 added batch orchestration, but parser-depth risk remains the main extraction limitation.

## Implemented M7 slice

1. BBT page diagnostics:
   - `parsed_entry_count`.
   - `truncated_entry_count`.
   - `page_type`.
   - `page_level`.
   - page `status`.
2. BBT index diagnostics:
   - `parsed_pages`.
   - `duplicate_entry_count`.
   - `truncated_entry_count`.
   - richer index status string.
3. NBT page diagnostics:
   - `parsed_entry_count`.
   - `truncated_entry_count`.
   - `page_type`.
   - `page_level`.
   - page `status`.
4. NBT index diagnostics:
   - `parsed_pages`.
   - `duplicate_entry_count`.
   - `truncated_entry_count`.
   - richer index status string.
5. Parser tests:
   - Complete BBT page parse diagnostics.
   - Truncated BBT page parse diagnostics.
   - Complete NBT page parse diagnostics.
   - Truncated NBT page parse diagnostics.

## Operational behaviour

- `pstd inspect` automatically surfaces richer BBT/NBT status strings because it reads index status.
- Extraction status strings include the richer BBT/NBT status without changing archive layout.
- Malformed/truncated entry counts are visible instead of being hidden behind a generic `root_page_parsed` status.

## Remaining parser-depth work

The next hardening slice should consider:

- Multi-level BBT/NBT page traversal.
- More precise BBT/NBT page-type validation.
- Block trailer validation.
- Property-context block loading from BBT lookups.
- Table-context row traversal from actual PST table nodes.
- Attachment subnode traversal.
- Synthetic/public fixtures that exercise deeper trees.

## Safety and privacy

Do not add private PST fixtures or extracted content. Any parser fixture must be synthetic, explicitly public, or non-sensitive and license-compatible.

## Definition of done

- M7 branch keeps CI green.
- BBT/NBT diagnostics are emitted.
- Parser diagnostics tests pass.
- Docs describe remaining parser-depth limitations accurately.
