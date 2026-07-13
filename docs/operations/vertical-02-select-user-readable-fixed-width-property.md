# Vertical 02: select the first user-readable fixed-width property

## Objective

Prevent Table Context bookkeeping properties from being selected as evidence of readable email content, then rerun the public PST fixture to reveal the next supported fixed-width property candidate.

## Baseline

The previous public-fixture run selected property tag `0x67f20003` and decoded values `45, 48, 51, 54`. Vertical 01 classified property identifier `0x67f2` as `PidTagLtpRowId`, an internal LTP row identity rather than message metadata.

## Implementation

`project_fixed_width_row_evidence` now filters descriptors through `classify_tc_property` before fixed-width evidence selection. Properties classified as `TableInternal` are excluded. Unknown properties remain eligible because their semantics have not been disproved, but they must not be assigned a user-facing MAPI meaning without separate validation.

The change remains fail closed:

- no table-internal property can be returned as readable evidence;
- if no supported non-internal candidate remains, the projection returns construction-failed evidence with no property metadata or values;
- payload and transport validation remain unchanged;
- no row payload bytes are serialized.

## Regression coverage

Tests prove that:

1. a supported unknown property is still decoded;
2. `PidTagLtpRowId` is ignored even when it has more distinct values than another candidate;
3. the non-internal candidate is selected instead;
4. a table containing only internal fixed-width candidates emits no partial evidence;
5. unavailable and malformed evidence paths continue to fail closed.

## Acceptance evidence required from CI

The public PST fixture should now produce one of two useful outcomes:

1. a different fixed-width property tag with raw and decoded values; or
2. explicit proof that no supported non-internal fixed-width property is present in the currently validated table.

The result must be recorded before assigning semantic meaning or choosing the next extraction milestone.

## Extraction impact

This milestone does not itself claim a newly readable subject, sender, timestamp, recipient, body, or attachment. It corrects candidate selection so that the next observed value is not known table bookkeeping.

## Next milestone decision

- If CI exposes a plausible non-internal fixed-width property, validate its MAPI identity and publish that one field end to end.
- If no supported candidate remains, stop extending fixed-width diagnostics and move directly to variable-width/HID/HNID resolution aimed at the first subject-like string.
