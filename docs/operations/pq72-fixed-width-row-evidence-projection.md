# PQ72 — validated fixed-width row evidence projection

## Repository finding

PQ71 added strict decoding for `PT_I2`, `PT_LONG`, `PT_BOOLEAN`, and `PT_I8`, but production code still lacked one fail-closed boundary that joined payload candidate resolution, validated row transport, and fixed-width decoding. Directly modifying the large `report_table_heaps` diagnostic surface in the same increment would combine transport, value selection, serialization, and fixture-output changes.

PQ72 therefore implements the smallest complete vertical decoding boundary: given the production inputs already available at the reporting call site, it returns decoded fixed-width evidence only after every existing validation layer succeeds.

## Implemented contract

`project_fixed_width_row_evidence`:

1. resolves row payload candidates through PQ69;
2. builds validated row transport through PQ68;
3. uses validated payload bytes, absolute offsets, and row width;
4. invokes the PQ71 fixed-width selector with TCINFO descriptors and bitmap masks;
5. returns raw hexadecimal and decoded scalar values only on full validation;
6. returns no partial evidence for unavailable, ambiguous, malformed, or unsupported cases.

Statuses are explicit:

- `tc_fixed_width_evidence_validated`
- `tc_fixed_width_evidence_unavailable`
- `tc_fixed_width_evidence_construction_failed`

Candidate and transport statuses remain separately visible for diagnosis.

## Regression coverage

Tests cover:

- an ordinal four-row `PT_LONG` slice decoding values `1,2,3,4`;
- payload-unavailable behavior with no partial evidence;
- invalid bitmap/descriptor evidence failing closed.

## Extraction impact

This increment adds an end-to-end library path from subnode payload discovery to decoded scalar evidence. Public extraction totals are unchanged because `TcHeapDiagnostic` serialization is intentionally deferred to the next integration increment.

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

PQ72 does not:

- select among ambiguous payload candidates;
- bypass row-address or row-bound validation;
- decode unsupported property types;
- publish row payload bytes;
- assign semantic MAPI property names;
- change message, body, recipient, or attachment counts.

## Revised next increment: PQ73

Integrate `project_fixed_width_row_evidence` into `report_table_heaps` and `TcHeapDiagnostic` without retaining payload bytes.

Requirements:

1. invoke the projection only for subnode-backed TC rows;
2. publish candidate, transport, and fixed-width evidence statuses;
3. publish selected property tag, data offset, data size, raw values, and decoded values only when validated;
4. emit empty values and no partial descriptor metadata on failure;
5. preserve all existing diagnostics and extraction counts;
6. add public-fixture assertions recording the actual selected tag and decoded values;
7. use that observed tag to choose the first semantically meaningful message-property vertical slice rather than guessing.
