# PQ22 Aggregate Table Counts

PQ22 implements issue #340.

## Scope

- Surface aggregate table parser counters in run status.
- Use the public PST artifact to decide whether PQ23 can map row values to property candidates.

## Expected decision

If row and value counters are non-zero, PQ23 should map table row values to property candidates. If counters remain zero, PQ23 should diagnose the real table row layout.
