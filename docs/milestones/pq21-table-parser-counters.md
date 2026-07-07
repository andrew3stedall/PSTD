# PQ21 Table Parser Counters

PQ21 follows the revised requirement from issue #338.

## Scope

- Propagate table parser counters into status.
- Preserve extraction output counts.
- Inspect the public PST artifact before revising PQ22.

## Expected decision

If parsed rows and values appear, PQ22 should move to row-to-property candidate mapping. If rows remain zero, PQ22 should inspect the real subnode table layout assumptions.
