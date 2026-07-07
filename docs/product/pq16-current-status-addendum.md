# PQ16 Current Status Addendum

PQ16 merged through PR #328 with green final-head CI.

Public fixture result:

| Metric | Value |
|---|---:|
| BBT entries | 50 |
| NBT entries | 63 |
| Folders | 11 |
| Messages | 1 |
| Attachments | 0 |
| Decoded supported message subnode blocks | 1 |
| Table-like message subnode layouts | 1 |

Current blocker: `message_subnode_table_payload_wiring`.

Related research: [PST Parser Research Summary](../research/pst-parser-research.md).

Next roadmap:

1. PQ17: table-context probe counters.
2. PQ18: table rows as property candidates.
3. PQ19: table-led folder/message membership counters.
