# PQ12 Payload Boundary Diagnostics

## Purpose

PQ12 follows PQ11 by determining whether the real public fixture message property payload contains a heap signature beyond the original bounded scan window.

## Implemented behaviour

- Extends heap signature and candidate scanning to a bounded 4096-byte diagnostic window.
- Separates three outcomes:
  - no heap signature found,
  - heap signature found without a valid page-map shape,
  - heap or BTH candidate parse failure.
- Preserves legacy flat-BTH fallback when no safe heap candidate is available.
- Aggregates PQ12 counters into run status.

## Public fixture result

CI #256 against `tests/fixtures/pst/sample.pst` produced:

| Metric | Value |
|---|---:|
| BBT entries | 50 |
| NBT entries | 63 |
| Folders | 11 |
| Messages | 1 |
| Attachments | 0 |
| Plausible property tags | 0 |
| Suspicious property keys | 70 |
| PQ12 no signature | 1 |
| PQ12 signature without page map | 0 |
| PQ12 heap candidate failed | 0 |
| PQ12 BTH candidate failed | 0 |

## Interpretation

The public fixture message payload still falls back to flat BTH interpretation, and no heap signature appears in the bounded 4096-byte payload window. This points away from heap-prefix scanning and toward selecting the wrong data block, missing subnode resolution, or misinterpreting the true node payload source.

## Next blocker

`pq12_next_blocker=payload_block_selection_or_subnode_resolution`
