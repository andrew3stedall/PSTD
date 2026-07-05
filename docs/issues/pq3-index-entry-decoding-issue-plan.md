# PQ3 Index Entry Decoding Issue Plan

## Parent

- #195 `[PQ3] BBT/NBT page entry decoding`

## Ordered issues

1. #196 Add index page diagnostics.
2. #197 Improve index entry decoding.
3. #198 Update conversion coverage docs.

## Execution order

1. Correct B-tree page metadata offsets.
2. Correct internal-page child reference decoding.
3. Preserve leaf-entry decoding for supported pages.
4. Surface page-level diagnostics in inspect output.
5. Update parser tests and conversion-focused docs.

## Done criteria

- Inspect JSON exposes page diagnostics.
- Supported page layouts decode entries.
- Unsupported or truncated layouts remain safe and classified.
- Snowflake and downstream loading are parked outside the active conversion-coverage path.
- CI passes before merge.
