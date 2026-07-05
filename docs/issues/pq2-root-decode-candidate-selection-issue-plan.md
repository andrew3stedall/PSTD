# PQ2 Root Decode Candidate Selection Issue Plan

## Parent

- #189 `[PQ2] Root decode correction and fixture classification`

## Ordered issues

1. #190 Add root candidate selection model.
2. #191 Add root selection regression tests.
3. #192 Update PQ2 parser-quality docs.

## Execution order

1. Extend the PQ1 root diagnostic model to support multiple candidates.
2. Select a safe candidate pair for traversal when available.
3. Keep all candidates visible in inspect JSON.
4. Add tests for safe selection and blocked traversal.
5. Update status, changelog, and operations docs.

## Done criteria

- Inspect JSON includes selected source and candidate list.
- BBT/NBT traversal receives selected safe roots only.
- No usable root pair is classified without attempting unsafe traversal.
- CI passes before merge.
