# PQ5 Message Table Discovery

## Purpose

PQ5 keeps the active work on PST conversion coverage. It improves message fidelity by separating decoded message nodes from folder/table/index nodes and by surfacing message-table evidence when the current NBT view exposes it.

PQ4 moved the public PST fixture from root-only folder output to decoded folder candidates. PQ5 addresses the next blocker: message rows should not be inflated by every decoded NBT entry.

## What changed

PQ5 adds deterministic NBT node-type classification for message and table evidence:

| Decoded node type | PQ5 classification |
|---:|---|
| `0x04` | normal message candidate |
| `0x08` | associated message candidate |
| `0x12` | contents table candidate |
| `0x13` | associated contents table candidate |
| `0x0d` | search contents table candidate |
| `0x11` | hierarchy table candidate |
| anything else | not emitted as a message row |

Message output now iterates only decoded normal/associated message candidates. Folder and table nodes are not emitted as message rows.

## Membership evidence

PQ5 derives owner-folder evidence for decoded table nodes by replacing the low node-type bits with the normal-folder node type. If that derived folder node was discovered by PQ4, the table candidate is counted as linked to a folder candidate.

This is evidence only. PQ5 does not yet decode table row payloads into authoritative folder-message membership. Message rows remain parented to the root unless and until a later milestone decodes table row membership.

## Extraction status counters

The extraction status includes PQ5 counters:

- `pq5_status`
- `pq5_message_candidates`
- `pq5_table_candidates`
- `pq5_linked_tables`
- `pq5_unlinked_tables`

Status records also call out when non-message NBT entries were excluded from message output.

## Known limitations after PQ5

PQ5 improves message candidate fidelity, but it does not claim complete message conversion coverage. It does not decode message table row payloads, body payloads, attachment payloads, recipients, or embedded messages.

## Next blocker

The next blocker is PQ6: property context and body coverage. Once message candidates are no longer inflated by folder/table entries, the next quality question is how many candidate messages have usable properties and bodies.
