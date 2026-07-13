# PQ68 — Bridge row resolution to validated transport evidence

## Context

PQ65 established bounded conversion from row references to absolute offsets. PQ66 coupled a complete payload with those offsets. PQ67 added fail-closed selection between direct byte offsets and ordinal row indices.

The remaining gap was a safe bridge between `TcSubnodeRowResolutionReport` and the PQ66 transport builder. The earlier PQ68 proposal also included public diagnostic publication and fixed-width value selection. That scope was reduced because combining transport construction, reporting, and value selection would make it harder to distinguish addressing defects from presentation defects.

## Implemented contract

`build_transport_from_row_resolution` accepts:

- one existing `TcSubnodeRowResolutionReport`;
- the row payload candidates resolved by the caller.

It returns one of three statuses:

- `tc_row_transport_validated`;
- `tc_row_transport_unavailable`;
- `tc_row_transport_construction_failed`.

Validated evidence is retained only when:

1. the resolver and caller agree on the payload count;
2. exactly one payload is available;
3. the payload length matches the resolver evidence;
4. the resolver status identifies exactly one validated addressing mode;
5. the PQ67 mode selector revalidates all row bounds;
6. the PQ66 transport builder succeeds.

Unavailable is limited to the case where the resolver and caller both report no payload. Ambiguous payloads, count mismatches, invalid layouts, malformed bounds, and payload-length mismatches fail closed.

## Evidence retained

On success the bridge returns:

- the complete row payload;
- validated absolute row offsets;
- validated row width;
- the selected direct or ordinal address mode.

On unavailable or failed results it returns no payload and no partial offsets. A construction failure includes a bounded diagnostic reason for tests and future reporting.

## Regression coverage

Tests cover:

- ordinal references `0,1,2,3` producing offsets `0,52,104,156` for a 208-byte payload;
- direct offsets `0,4,8` remaining unchanged;
- no resolved payload producing `unavailable`;
- multiple payloads producing `construction_failed`;
- payload-length mismatch producing `construction_failed`;
- no partial evidence in any failed state.

## Extraction impact

PQ68 does not change extraction totals or public diagnostic output. The validated public-fixture baseline remains:

- 14 TCINFO descriptors;
- 4 rows;
- 52-byte row width;
- 208-byte row payload;
- absolute offsets `0,52,104,156`;
- bitmap masks `11111011000000` for each row;
- 1 extracted message;
- 2 body payloads;
- 0 attachments.

The improvement is structural: downstream reporting can now consume a single object that has passed resolver-count, mode-selection, offset, payload-length, and row-bound validation.

## Safety boundary

This increment does not:

- interpret property tags;
- infer semantic property presence from bitmap bits;
- select a fixed-width descriptor;
- decode row bytes;
- follow HID or HNID references;
- alter message or attachment extraction.

## Proposed PQ69

PQ69 should integrate this bridge into the existing row-resolution call site so the unique matching payload is supplied without reconstructing it elsewhere.

Requirements:

1. invoke the bridge immediately after `resolve_subnode_row_storage` where the matching row payload is still available;
2. publish only the transport status and bounded metadata initially;
3. retain no payload bytes in serialized diagnostics;
4. emit absolute offsets only for `tc_row_transport_validated`;
5. preserve all existing descriptor, bitmap, row-layout, and extraction evidence;
6. add public-fixture assertions for ordinal mode and offsets `0,52,104,156`;
7. continue to avoid selecting or interpreting property values.

The fixed-width selector should remain deferred until the call-site integration is verified by the public fixture.
