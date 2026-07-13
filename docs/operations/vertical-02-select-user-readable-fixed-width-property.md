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

## Public fixture acceptance evidence

GitHub Actions run 515 completed successfully on commit `aec65bc006a08d6cccdf2ae4a11c71df43704083`.

The public PST fixture selected the next non-internal fixed-width property:

- property tag: `0x0c150003`;
- property identifier: `0x0c15`;
- property type: `PT_LONG`;
- raw values: `01000000`, `01000000`, `02000000`, `02000000`;
- decoded values: `1`, `1`, `2`, `2`;
- affected table rows: `4`;
- candidate status: `tc_row_payload_candidates_resolved`;
- transport status: `tc_row_transport_validated`;
- evidence status: `tc_fixed_width_evidence_validated`;
- failure reason: none.

This is a real extraction change: the production fixture no longer reports the known internal `PidTagLtpRowId` values and instead exposes a distinct supported property. Semantic naming is intentionally deferred until the property identifier is validated against the MAPI specification.

## Before-versus-after extraction result

| Measure | Before | After |
|---|---:|---:|
| Messages discovered | 1 | 1 |
| Messages extracted | 1 | 1 |
| Body payload records | 2 | 2 |
| Recipient records emitted | 0 | 0 |
| Attachments extracted | 0 | 0 |
| EML files emitted | 0 | 0 |
| Output bytes | 33,739 | 33,739 |
| Selected fixed-width property | `0x67f20003` | `0x0c150003` |
| Selected decoded values | `45,48,51,54` | `1,1,2,2` |

No message, body, attachment, archive, or output-byte regression was observed.

## Extraction impact

This milestone does not yet claim a readable subject, sender, timestamp, recipient, body, or attachment. It produces observable non-internal property data and removes a confirmed false-positive internal property from production selection.

## Next milestone decision

Validate property identifier `0x0c15` against the authoritative MAPI property specification. If it is a user-meaningful property, publish its semantic name and interpreted values. If it is table-specific or recipient-specific rather than message metadata, use that evidence to move directly into the corresponding vertical extraction path instead of adding another selector or diagnostic layer.
