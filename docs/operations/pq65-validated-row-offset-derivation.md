# PQ65 — Validated row-offset derivation

## Evidence basis

PQ64 introduced a bounded selector for one four-byte TCINFO field, but the reporting layer still lacks a reusable way to convert validated row references into absolute byte offsets. The public fixture currently uses ordinal references `0:1:2:3` for four 52-byte rows in a 208-byte payload.

## Scope correction

Directly publishing fixed-width values would combine row-address interpretation, payload transport, candidate selection and diagnostic formatting. PQ65 isolates the row-address prerequisite first.

## Requirements

`derive_validated_row_offsets` must:

- accept either direct byte offsets or ordinal row indices;
- require a non-zero validated row width;
- use checked multiplication and addition;
- require every complete row to fit inside the resolved payload;
- require derived offsets to be strictly increasing;
- return no partial offsets when any reference fails validation.

## Validation evidence

Focused tests cover:

- direct offsets `0,52,104,156`;
- ordinal references `0,1,2,3` resolving to the same offsets;
- out-of-bounds ordinal rows;
- duplicate or non-increasing direct offsets.

The full repository workflow must remain green, including the public PST fixture. Extraction counts and PQ63 descriptor evidence must remain unchanged.

## Safety boundary

PQ65 does not retain row payload bytes in reporting, select a descriptor, emit raw row values, interpret property tags, or assign semantic presence to bitmap states.

## Proposed PQ66

PQ66 should transport the single resolved row payload and validated absolute row offsets through `TcSubnodeRowResolutionReport`. It should derive offsets using this helper and fail closed when the row mode is ambiguous. It should not yet publish selected values.

PQ67 can then connect the PQ64 selector to `TcHeapDiagnostic` using transported payload bytes, offsets, masks, descriptors, row width and fixed-data boundary.
