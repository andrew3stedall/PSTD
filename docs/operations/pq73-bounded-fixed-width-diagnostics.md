# PQ73 — bounded fixed-width diagnostic projection

## Repository finding

PQ72 completed the validated path from subnode row payload discovery to decoded fixed-width scalar evidence. The next proposed increment was to write that evidence directly into `TcHeapDiagnostic` and the long-lived `pq42` status string.

Review of the reporting code showed that this would mix two concerns in one change: defining the fail-closed public representation and mutating the production reporting surface. The existing status string is consumed by fixture assertions and downstream run reporting, so publishing optional evidence without first defining exact empty-state behavior would risk partial or ambiguous diagnostics.

PQ73 therefore establishes the bounded public representation. The next increment can integrate one tested type into `TcHeapDiagnostic` rather than reproducing projection rules inside `report_table_heaps`.

## Implemented contract

`build_fixed_width_diagnostic` converts a `TcFixedWidthProjectionReport` into a payload-free diagnostic object.

Validated evidence publishes:

- candidate status;
- transport status;
- fixed-width evidence status;
- selected property tag;
- data offset and data size;
- raw hexadecimal row values;
- decoded row values.

Unavailable or failed evidence publishes statuses and a bounded failure reason, but no property tag, offset, size, raw values, or decoded values.

`status_fragment` uses stable field names and replaces semicolons in externally derived status and failure text so it can be embedded safely in the existing semicolon-delimited progress output.

## Regression coverage

Tests cover:

- validated scalar evidence retaining property metadata and raw/decoded values;
- failed evidence exposing no partial descriptor or row-value metadata;
- stable payload-free status formatting.

## Extraction impact

No extraction totals change in PQ73. The improvement is that fixed-width evidence now has one exact public representation ready for production integration.

Current baseline:

| Measure | Baseline |
|---|---:|
| TCINFO descriptors | 14 |
| Rows | 4 |
| Row width | 52 bytes |
| Row payload | 208 bytes |
| Absolute offsets | `0,52,104,156` |
| Bitmap masks | `11111011000000` × 4 |
| Messages extracted | 1 |
| Body payloads | 2 |
| Attachments | 0 |

## Safety boundary

PQ73 does not:

- serialize row payload bytes;
- select among ambiguous payload candidates;
- publish partial property metadata after failure;
- assign semantic MAPI property names;
- change message, body, recipient, or attachment counts;
- modify the existing `pq42` reporting surface.

## Revised next increment: PQ74

Integrate `build_fixed_width_diagnostic` into `report_table_heaps` and `TcHeapDiagnostic`.

Requirements:

1. invoke `project_fixed_width_row_evidence` only for subnode-backed TC rows;
2. convert its result only through `build_fixed_width_diagnostic`;
3. retain the diagnostic object on `TcHeapDiagnostic`;
4. append its stable status fragment to the existing heap diagnostic output;
5. use an explicit unavailable diagnostic for non-subnode rows and failed TC heap resolution;
6. serialize no row payload bytes;
7. preserve all existing descriptor, bitmap, layout, and extraction fields;
8. run the public PST fixture and record the actual selected property tag and raw/decoded values;
9. choose the first semantic property milestone from observed evidence rather than assuming the selected tag represents a subject, sender, or timestamp.