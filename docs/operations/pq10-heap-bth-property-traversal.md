# PQ10 Heap BTH Property Traversal

## Purpose

PQ10 adds a conservative heap-on-node property-context traversal path before the existing flat BTH fallback.

## Implemented behaviour

- Heap-on-node header and page-map parsing.
- One-based heap HID allocation resolution.
- Heap-backed property-context BTH parsing.
- MAPI property tag reconstruction from property ID plus property type.
- Local heap value dereference through HNIDs when possible.
- Traversal selection reporting in `run_summary.status`.

## Public fixture result

CI #235 against `tests/fixtures/pst/sample.pst` produced:

| Metric | Value |
|---|---:|
| BBT entries | 50 |
| NBT entries | 63 |
| Folders | 11 |
| Messages | 1 |
| Attachments | 0 |
| Plausible property tags | 0 |
| Suspicious property keys | 70 |
| Heap BTH contexts | 0 |
| Legacy flat BTH contexts | 1 |
| Unknown traversal contexts | 0 |

## Interpretation

The public fixture still uses the flat BTH fallback for the one message candidate. PQ10 narrows the next repair target to heap/HN header or BTH-root detection for that real payload.

## Next blocker

`pq10_next_blocker=heap_hn_header_or_bth_root_detection`
