# PQ19 Table Membership Counters

## Goal

Measure table-led hierarchy and contents membership availability after PQ18.

## Implemented signal

PQ19 adds public progress artifact counters for:

- `pq19_hierarchy_table_rows`
- `pq19_contents_table_rows`
- `pq19_table_linked_folders`
- `pq19_table_linked_messages`
- `pq19_next_blocker`

## Current public-fixture expectation

PQ18 found zero table candidate rows, so PQ19 should show zero hierarchy rows, zero contents rows, zero table-linked folders, and zero table-linked messages.

## Boundary

PQ19 keeps folder/message output unchanged until decoded table rows exist. It does not broaden raw NBT heuristics.
