# PQ49 — prioritize the actionable subnode row-storage diagnosis

## Evidence from PQ48

The public PST fixture reached one real table-context heap:

- probes: 1
- decoded payloads: 4
- resolved table heaps: 1
- failed table heaps: 0
- columns: 14
- row references: 4
- subnode-backed row heaps: 1
- in-bounds row references: 0
- out-of-bounds row references: 0

The diagnostic reported `tc_heap_index_unresolved` even though the same report proved that the row matrix is NID-backed and therefore cannot be bounds-checked until the subnode row payload is resolved. The optional index allocation was masking the actionable blocker.

## Revised requirement

PQ49 changes diagnostic precedence only. Once the row index is decoded, NID-backed row storage must be reported as `tc_heap_rows_require_subnode_resolution` before an unresolved optional index allocation.

This does not materialise rows and does not claim an extraction-volume improvement. It makes the next implementation requirement evidence-driven rather than directing work toward the wrong allocation.

## Changes

- Reordered table-context status selection so subnode-backed row storage takes precedence over `index_hid` resolution.
- Added a regression test where the row matrix is NID-backed and the optional index HID is unresolved.
- Preserved the row-matrix HNID in `TcHeapResolutionReport` for the following resolver work.

## Before versus after

| Measure | Before PQ49 | After PQ49 |
|---|---|---|
| Real table heap reached | 1 | 1 expected |
| Columns discovered | 14 | 14 expected |
| Row references discovered | 4 | 4 expected |
| Subnode-backed row heaps | 1 | 1 expected |
| Primary diagnosis | `tc_heap_index_unresolved` | `tc_heap_rows_require_subnode_resolution` |
| Messages, bodies, attachments, EMLs | Existing baseline | Unchanged expected |

## Evidence required from CI

The public fixture should retain the PQ48 extraction totals while changing the table diagnostic status to `tc_heap_rows_require_subnode_resolution`. Any change to folder, message, body, attachment, or output-byte totals is unexpected and must be investigated before merge.

## Proposed PQ50

Resolve the table row-matrix NID against the decoded recursive subnode payload set, then validate the four row references against the resolved row-data byte length. Do not decode column values until row offsets and fixed row width are proven valid.
