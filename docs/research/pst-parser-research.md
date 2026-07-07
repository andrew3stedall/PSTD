# PST Parser Research Summary

## Decision

Future PSTD conversion-quality work should be table-led rather than broad NBT-heuristic-led.

## Why

Microsoft PST files are layered: header/root, BBT/NBT, Heap-on-Node, BTree-on-Heap, Property Contexts, Table Contexts, and then messaging semantics. Folder-message-recipient-attachment fidelity depends on table contexts and row mappings, not just generic node enumeration.

## Current PSTD position

PSTD has reached PQ16 with the public fixture showing:

- 50 BBT entries.
- 63 NBT entries.
- 11 folders.
- 1 message.
- 0 attachments.
- 1 decoded supported message subnode block.
- 1 table-like message subnode layout.

The current blocker is `message_subnode_table_payload_wiring`.

## Recommended near-term milestones

| Milestone | Direction |
|---|---|
| PQ17 | Wire the table-like message subnode into table-context parse/probe counters. |
| PQ18 | Use table rows and values as alternate message property candidates and measure selected/plausible property lift. |
| PQ19 | Shift folder/message membership toward table-led semantics and expose hierarchy/contents table counters. |

## Later roadmap themes

- Complete HN/BTH/PC decoding and HNID dereferencing.
- Add named-property map diagnostics and resolution.
- Add body/codepage provenance for Unicode, String8, HTML, and RTF sources.
- Add recipient and attachment table/subnode consistency checks.
- Keep clean parsing and recovery mode separate.
- Add differential fixture validation against external parser outputs where practical.

## Useful counters

| Group | Counters |
|---|---|
| Structural | `header_magic_ok`, `magic_client_ok`, `pst_version`, `pst_variant`, `bbt_entries`, `nbt_entries`, `duplicate_bids`, `duplicate_nids` |
| HN/BTH/PC | `hn_blocks`, `hn_signature_failures`, `heap_allocations`, `bth_count`, `hnid_heap_refs`, `hnid_subnode_refs`, `pc_properties_total` |
| Tables | `tc_count`, `table_like_subnode_layouts`, `tc_parse_attempts`, `tc_parse_successes`, `tc_parse_failures`, `tc_columns`, `tc_rows` |
| Messaging | `folders_direct_linked`, `messages_direct_linked`, `recipient_table_missing`, `attachment_table_missing_when_flagged` |
| Text/body | `string8_props_total`, `string_decode_failures`, `body_plain_present`, `body_html_present`, `body_rtf_present` |
| Recovery | `recovery_mode_enabled`, `orphan_nodes`, `orphan_subnodes`, `carved_candidates`, `soft_fail_objects`, `hard_fail_objects` |

## Implementation rule

Do not broaden NBT heuristics as the next step. Prioritise HN/BTH/PC/TC correctness, table membership, text provenance, and recovery separation.
