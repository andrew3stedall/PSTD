# PQ50 — resolve NID-backed table row storage

## Evidence from PQ49

PQ48 reached one real table-context heap with 14 columns and four row references. PQ49 corrected the primary diagnosis to `tc_heap_rows_require_subnode_resolution`, confirming that the next blocker is the row-matrix NID rather than the optional table index allocation.

## Revised requirement

Resolve a table context's NID-backed row matrix only when the decoded recursive payload set contains one unambiguous Unicode SLBLOCK entry for the requested NID and one matching data payload BID. Bounds-check row references against that payload length, but do not decode column values yet.

The resolver must distinguish:

- missing NID entry;
- NID entry whose data BID was not decoded;
- ambiguous duplicate entries or payloads;
- resolved rows with all references in bounds;
- resolved rows with one or more references out of bounds.

## Changes

- Added a bounded Unicode SLBLOCK NID-to-data-BID resolver.
- Matches the row-matrix NID to an already decoded recursive payload.
- Validates row references against the resolved payload byte length.
- Carries the row NID, entry count, resolved payload count, row byte length, and bounds totals into table diagnostics.
- Preserves the existing extraction path and does not materialise unverified row values.

## Before versus after

| Measure | Before PQ50 | After PQ50 target |
|---|---:|---:|
| Real table heaps reached | 1 | 1 |
| Columns discovered | 14 | 14 |
| Row references discovered | 4 | 4 |
| NID-backed row heaps | 1 | 1 |
| Row-matrix NID lookup | Not implemented | Implemented |
| Row payload byte length | 0 | Reported when uniquely resolved |
| In/out-of-bounds row references | 0 / 0 | Derived from resolved row payload |
| Messages, bodies, attachments, EMLs | Existing baseline | Expected unchanged |

## Evidence required from CI

The public fixture must retain the existing folder, message, body, attachment, EML, and output-byte totals. The table diagnostic must expose `rows_nid`, matching SLBLOCK entries, resolved row payloads, row-data byte length, and row-reference bounds counts.

A missing or ambiguous NID association is a valid evidence result and must not be treated as successful row resolution.

## Proposed PQ51 decision boundary

- If the row payload resolves and all four references are in bounds, derive and validate fixed row width and column boundaries without decoding values.
- If the payload resolves with out-of-bounds references, investigate whether row references are byte offsets or row indices before proceeding.
- If the NID is missing or ambiguous, inspect the exact SLBLOCK entry representation and recursive payload association before changing table parsing.
