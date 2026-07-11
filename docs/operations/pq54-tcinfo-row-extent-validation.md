# PQ54 — validate TCINFO row extents

## Evidence from PQ53

GitHub Actions run #434 passed the full CI matrix. The public PST fixture reported `tc_subnode_rows_ordinal_index_validated_52` for four ordinal row references over a 208-byte payload. Extraction totals remained stable at 11 folders, one extracted message, two body payloads, zero attachments, one tar shard, and 29,806 output bytes.

## Revised requirement

PQ54 validates structural bounds before reading any row bytes:

1. Preserve TCINFO data-region boundaries in the heap resolution report.
2. Calculate the maximum `data_offset + data_size` across all column descriptors.
3. Calculate presence-bitmap length as `ceil(column_count / 8)` and its end offset from the final TCINFO boundary.
4. Compare the final region boundary, maximum column extent, and bitmap end with the proven row width.
5. Publish a separate row-layout status without replacing the row-index interpretation status.
6. Decode no bitmap bits, columns, or property values.

## Expected fixture evidence

The fixture should retain the 52-byte ordinal-row status and additionally report either `tc_row_layout_extents_validated_52` or an exact out-of-bounds result with the boundaries and extents visible. Existing extraction totals must remain unchanged.

## Safety boundary

A successful bounds check establishes only that structural regions fit inside each row. It does not establish property-type decoding, nullability, variable-value indirection, or semantic correctness.

## Proposed PQ55

If all extents fit, decode only the presence bitmap and report per-row present/absent counts without reading property values. If any extent overflows, inspect the exact TCINFO boundary interpretation before accessing row bytes.
