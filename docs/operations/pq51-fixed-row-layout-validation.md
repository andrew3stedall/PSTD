# PQ51 — validate fixed table-row layout

## Evidence from PQ50

The public PST fixture resolved the row-matrix NID `0x809f` through one SLBLOCK entry to one 208-byte payload. All four row references were in bounds, and the table retained 14 columns. Existing extraction totals remained stable at 11 folders, one message, two body payloads, zero attachments, one tar shard, and 29,806 output bytes.

## Revised requirement

Before decoding any table values, prove that the resolved row payload has a fixed-width layout and that the TCINFO data-region boundary and every column extent fit within that width. Reject empty, non-increasing, variable-width, and overflowing layouts.

## Changes

- Derive row segments from consecutive row references plus the final payload tail.
- Require all row segments to have one non-zero width.
- Expose the inferred row width and fixed-width result.
- Validate the final TCINFO data-region boundary against the inferred width.
- Validate the maximum `data_offset + data_size` column extent against the inferred width.
- Preserve the safety boundary: no column values are decoded or materialised.

## Expected fixture evidence

The four references and 208-byte payload are expected to produce four 52-byte rows. The diagnostic should report `row_width=52`, `fixed_width=1`, `regions_in_bounds=1`, `columns_in_bounds=1`, and `status=tc_subnode_rows_layout_validated`.

## Failure meanings

- `tc_subnode_rows_variable_width`: references do not partition the payload into equal non-zero rows.
- `tc_subnode_rows_layout_out_of_bounds`: TCINFO region or column extents exceed the inferred row width.
- Existing missing, ambiguous, payload-missing, and reference-out-of-bounds states remain unchanged.

## Proposed PQ52 decision boundary

- If the real fixture validates a 52-byte row width and all extents fit, decode only the row presence bitmap and report present/absent column counts without interpreting values.
- If widths vary, inspect whether row references are indices or require a separate row-size source.
- If column or region extents overflow, inspect the exact TCINFO boundary semantics before decoding any row bytes.
