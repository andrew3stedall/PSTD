# PQ11 Heap Root Detection

## Purpose

PQ11 follows PQ10 by checking why the public fixture message payload still uses flat BTH fallback instead of heap-backed property-context traversal.

## Implemented behaviour

- Validates heap signatures before accepting heap candidates.
- Scans a bounded payload prefix for heap candidates.
- Supports bounded indexed heap BTH traversal.
- Reports heap probe outcomes in extraction status.

## Public fixture result

CI #246 against `tests/fixtures/pst/sample.pst` produced:

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
| PQ11 offset heap contexts | 0 |
| PQ11 candidate not found | 1 |
| PQ11 candidate heap failed | 0 |
| PQ11 candidate BTH failed | 0 |

## Interpretation

The real public fixture message payload does not show a valid heap signature in the bounded scan window. This points away from simple heap root offset detection and toward payload selection or payload prefix interpretation.

## Next blocker

`pq11_next_blocker=heap_signature_or_block_payload_prefix_detection`
