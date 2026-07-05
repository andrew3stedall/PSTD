# PQ3 Index Entry Decoding

## Purpose

PQ3 focuses only on PST conversion coverage. It moves PSTD from selected root pages into safer B-tree page entry decoding.

PQ2 proved that the 271 KB public PST fixture can reach in-bounds root pages. PQ3 fixes the next blocker: B-tree page metadata and internal-page child references were being decoded from the wrong offsets.

## What changed

PST B-tree page metadata is read from the page metadata area near the end of each 512-byte page:

| Field | Offset |
|---|---:|
| Entry count | 488 |
| Entry capacity | 489 |
| Entry size | 490 |
| Page level | 491 |
| Page trailer start | 496 |

Entries are read from the page body starting at byte 0.

Internal pages now produce child page references from the child BREF offset field. Leaf pages produce BBT or NBT entries.

## Inspect output

`pstd inspect --json` now includes:

- `bbt_page_diagnostics[]`
- `nbt_page_diagnostics[]`

Each page diagnostic includes:

- `source_offset`
- `entry_count`
- `entry_capacity`
- `entry_size`
- `parsed_entry_count`
- `truncated_entry_count`
- `page_type`
- `page_level`
- `status`

## How to interpret results

If root diagnostics are green but page diagnostics show zero parsed entries, focus on page layout decoding before folder, message, body, or attachment work.

If BBT and NBT entries are non-zero, the next blocker moves to folder hierarchy and message table discovery.

## Active conversion roadmap

Snowflake, UI, search, and downstream loading are parked. The active work is PST conversion coverage:

1. PQ3: index page entry decoding.
2. PQ4: folder hierarchy discovery.
3. PQ5: message table discovery.
4. PQ6: property context and body coverage.
5. PQ7: attachment and recipient coverage.
6. PQ8: fixture corpus and ground-truth comparison.
