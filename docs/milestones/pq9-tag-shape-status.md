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

- Carries PQ8 diagnostics on `PropertyContext`.
- Adds compact PQ9 tag-shape status to message extraction status.
- Aggregates PQ9 counters into `run_summary.status`.
- Emits `pq9_next_blocker` so the next milestone can be selected from measured fixture evidence.

## Non-goals

- No selected dictionary expansion.
- No heap/BTH traversal repair.
- No body, attachment, or recipient expansion.

## Next milestone

The next milestone should follow the measured `pq9_next_blocker` from the public fixture artifact.
