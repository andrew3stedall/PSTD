# PQ4 Folder Hierarchy Discovery

## Purpose

PQ4 keeps the post-v1 focus on PST conversion coverage. It moves PSTD beyond a single synthetic root folder row by deriving folder candidates from decoded NBT entries.

PQ3 proved that the public PST fixture can decode BBT/NBT entries. PQ4 uses that decoded index evidence to classify folder-like nodes and surface deterministic folder rows before moving on to message table fidelity.

## What changed

Folder discovery now uses the low node-type bits from decoded NBT node IDs:

| Decoded node type | PQ4 classification |
|---:|---|
| `0x02` | normal folder candidate |
| `0x03` | search folder candidate |
| anything else | not a folder candidate |

For each folder candidate, extraction attempts to load the node property context. When the property context is available, PQ4 uses selected properties such as `PR_DISPLAY_NAME`, `PR_CONTENT_COUNT`, and `PR_CONTENT_UNREAD` to populate the folder and inventory rows. When the property context is unavailable, PQ4 still emits a deterministic node-derived row and records a recoverable status issue.

## Output behaviour

`data/folders.jsonl` and `_pstfast/folder_inventory.jsonl` now contain:

1. The root folder row.
2. One row for each decoded folder candidate, parented to the root until full folder-table hierarchy decoding is implemented.

Each emitted folder row includes source/status evidence in the `status` or `inventory_status` field. Extraction summary status also includes PQ4 counters:

- `pq4_status`
- `pq4_folder_candidates`
- `pq4_folder_property_loaded`
- `pq4_folder_property_unavailable`
- `folders_discovered`

## Fallback behaviour

If no decoded folder candidates exist, PSTD keeps the safe root-only output and reports `root_only_no_decoded_folder_candidates`.

If a decoded folder candidate exists but its property context cannot be loaded, PSTD emits a deterministic fallback folder row named from the node identity and records `pq4_folder_property_context_unavailable`.

## Known limitations after PQ4

PQ4 does not claim complete folder hierarchy fidelity. It does not yet decode folder table parent-child relationships, message table membership, recipients, body payloads, or attachment payload coverage. Folder candidates are parented to the root until the next conversion-quality milestones decode table-level relationships.

## Next blocker

The next blocker is PQ5: message table discovery. PQ5 should stop relying on raw NBT entries as message metadata candidates and instead map messages through folder/message table evidence.
