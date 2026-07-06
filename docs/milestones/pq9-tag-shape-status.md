# PQ9 Property Tag-Shape Status

## Goal

Surface the PQ8 property-context tag-shape counters through extraction status and the public PST artifact.

## Scope

PQ9 is bounded to issues #281 through #285.

| Issue | Scope | Status |
|---:|---|---|
| #281 | Parent scope | Implemented in this PR |
| #282 | Extraction-level tag-shape aggregation | Implemented |
| #283 | Public progress decision signal | Implemented |
| #284 | Docs and public PST progress | Implemented; final artifact logged before merge |
| #285 | Validation and merge | Completed after final-head CI is green |

## Delivered behaviour

- Derives PQ9 tag-shape status from property-value parse statuses without changing the public `PropertyContext` struct literal shape.
- Adds compact PQ9 tag-shape status to message extraction status.
- Aggregates PQ9 counters into `run_summary.status`.
- Emits `pq9_next_blocker` so the next milestone can be selected from measured fixture evidence.

## Public fixture result

CI #218 against `tests/fixtures/pst/sample.pst` produced:

| Metric | Value |
|---|---:|
| BBT entries | 50 |
| NBT entries | 63 |
| Folders | 11 |
| Messages | 1 |
| Attachments | 0 |
| Plausible property tags | 0 |
| Suspicious property keys | 70 |
| Byte-swapped selected properties | 0 |

## Non-goals

- No selected dictionary expansion.
- No heap/BTH traversal repair.
- No body, attachment, or recipient expansion.

## Next milestone

The next milestone should address `heap_bth_layout_traversal`: real heap-on-node/BTH/property-context traversal repair before additional dictionary, body, attachment, or recipient work.
