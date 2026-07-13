# PQ67: validated row address mode selection

## Outcome

PQ67 adds a reusable fail-closed selector that converts already validated row-layout evidence into exactly one `TcRowAddressMode`.

The selector accepts:

- row references;
- validated row width;
- resolved row payload length;
- direct fixed-width layout evidence;
- ordinal-index layout evidence.

It returns either `DirectOffsets` or `OrdinalIndices` only when exactly one layout mode is asserted and the resulting offsets still satisfy the PQ65 bounds invariant.

## Scope correction

The previous PQ67 proposal combined mode selection, unique payload resolution, row transport construction, reporting status, and diagnostic publication. That would allow a mode-selection defect to produce plausible but incorrectly addressed row bytes.

PQ67 therefore isolates and tests the address-mode contract before transport is attached to `TcSubnodeRowResolutionReport`.

## Validation rules

The selector fails closed when:

- neither layout mode is validated;
- both layout modes are asserted;
- row references are missing;
- row width is unavailable;
- derived offsets overflow;
- any complete row exceeds the resolved payload;
- derived offsets are not strictly increasing.

The selected mode is revalidated through `derive_validated_row_offsets`; the layout flags alone are not trusted.

## Tests

Coverage includes:

- validated direct offsets;
- validated ordinal indices;
- unavailable layout evidence;
- ambiguous layout evidence;
- a selected ordinal mode whose final row exceeds the payload.

## Extraction evidence

PQ67 does not change extraction output or diagnostic volume. The current validated fixture evidence remains:

- 14 TCINFO descriptors;
- 4 rows;
- 52-byte row width;
- 208-byte row payload;
- absolute row offsets `0,52,104,156`;
- bitmap masks `11111011000000` for each row;
- 1 message extracted;
- 2 body payloads;
- 0 attachments.

## Safety boundary

PQ67 does not:

- retain row payload bytes in reporting;
- publish absolute offsets;
- select a fixed-width descriptor;
- interpret row bytes;
- assign semantic meaning to bitmap states;
- alter extraction counts.

## Proposed PQ68

PQ68 should integrate the mode selector and `build_validated_row_transport` into `resolve_subnode_row_storage`.

Requirements:

1. require exactly one resolved row payload;
2. select the mode only through `select_validated_row_address_mode`;
3. construct transport evidence only through the PQ66 builder;
4. retain payload and offsets only on complete validation;
5. expose `tc_row_transport_validated`, `tc_row_transport_unavailable`, or `tc_row_transport_construction_failed`;
6. emit no payload or offsets on failure;
7. preserve all bitmap, descriptor, row-layout, and extraction evidence;
8. avoid selecting or interpreting property values.

PQ69 can then invoke the PQ64 fixed-width selector using the transported descriptors, masks, payload, offsets, row width, and fixed-data boundary.
