# PQ1 Root Decode Diagnostics

## Goal

Make PST root/header decoding failures explicit and actionable after the completed v1 lane.

## Why this exists

Public PST fixture CI tests completed safely but could not reach folder/message traversal because decoded root offsets were outside the file bounds. PQ1 records that condition in inspect output so the next parser-quality milestone can distinguish fixture issues from header/root decoding issues.

## Scope

- Add safe root diagnostics to header summaries.
- Surface a top-level inspect diagnostic condition.
- Add tests for out-of-bounds root offsets.
- Document how to run and interpret PQ1 diagnostics.

## Out of scope

- Full root decoding repair.
- New extraction features.
- Snowflake ingestion implementation.

## Acceptance criteria

- Inspect JSON includes root diagnostics.
- Impossible root offsets classify as `root_offsets_out_of_bounds`.
- Regression tests cover this failure class without private data.
- Docs identify PQ1 as the current parser-quality blocker before Snowflake ingestion work.
