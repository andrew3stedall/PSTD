# PQ70 — bounded row transport metadata projection

## Repository finding

PQ69 exposed a bounded row-payload candidate resolver. PQ68 already builds validated row transport evidence from a `TcSubnodeRowResolutionReport` and supplied payload slices. The production `report_table_heaps` call site has both the complete decoded payload set and the row-resolution report, but directly modifying public diagnostics would combine three concerns in one change:

1. row-payload candidate resolution;
2. validated transport construction;
3. serialization and progress-status formatting.

PQ70 isolates the second-to-third boundary. It creates a metadata-only projection that invokes the PQ69 resolver and PQ68 bridge, retains no row payload bytes, and emits no partial mode, width, or offsets when validation is unavailable or fails.

## Implemented contract

`resolve_row_transport_metadata` accepts:

- the decoded payload set;
- the row NID;
- the existing `TcSubnodeRowResolutionReport`.

It returns:

- payload-candidate status;
- transport status;
- address mode only for validated transport;
- row width only for validated transport;
- absolute row offsets only for validated transport;
- a bounded failure reason for construction failures.

Address modes are serialized as `direct_offsets` or `ordinal_indices`. Row payload bytes are never included in the metadata structure.

## Fail-closed behavior

Unavailable or failed transport always returns:

- `address_mode = None`;
- `row_width = None`;
- an empty absolute-offset vector.

Ambiguous payload candidates remain ambiguous and are not heuristically selected. Candidate and transport statuses are both retained because they describe distinct failure boundaries.

## Regression coverage

Tests cover:

- the public-fixture ordinal pattern `0,1,2,3` with width `52`, producing offsets `0,52,104,156`;
- direct offsets `0,4,8` with width `4`;
- unavailable payload evidence;
- ambiguous payload candidates with no partial transport metadata.

## Extraction evidence

PQ70 intentionally does not change extraction volume or public diagnostic output.

| Measure | Validated baseline |
|---|---:|
| TCINFO descriptors | 14 |
| Rows | 4 |
| Row width | 52 bytes |
| Row payload | 208 bytes |
| Expected absolute offsets | `0,52,104,156` |
| Bitmap masks | `11111011000000` × 4 |
| Messages extracted | 1 |
| Body payloads | 2 |
| Attachments | 0 |

## Safety boundary

PQ70 does not:

- serialize row payload bytes;
- select or interpret a property value;
- assign semantic meaning to bitmap bits;
- resolve HID/HNID-backed values;
- change extraction counts.

## Proposed PQ71

Integrate `resolve_row_transport_metadata` into `report_table_heaps` and `TcHeapDiagnostic`.

Requirements:

1. invoke the metadata projection only for subnode-backed rows;
2. publish candidate and transport status for every such attempt;
3. publish address mode, row width, and offsets only when transport is validated;
4. emit `none`, zero, and an empty offset vector for unavailable or failed transport;
5. serialize no payload bytes;
6. preserve descriptor, bitmap, layout, and extraction diagnostics;
7. add public-fixture assertions for `ordinal_indices`, width `52`, and offsets `0,52,104,156`;
8. continue to defer fixed-width value selection and semantic interpretation.
