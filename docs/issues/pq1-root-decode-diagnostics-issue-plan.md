# PQ1 Root Decode Diagnostics Issue Plan

## Parent

- #184 `[PQ1] PST root decode diagnostics`

## Ordered issues

1. #185 Add safe PST root diagnostic model.
2. #186 Add fixture regression tests for root bounds diagnostics.
3. #187 Document PQ1 real-PST quality lane.

## Execution notes

PQ1 is a post-v1 quality milestone. It does not expand extraction features. It makes root traversal blockers visible and classifiable before additional parser work or Snowflake ingestion planning proceeds.

## Done criteria

- Inspect JSON includes a root diagnostic condition.
- Header summary includes per-root bounds diagnostics.
- Tests cover impossible root offsets without private PST data.
- Documentation explains how to interpret results.
- CI passes before merge.
