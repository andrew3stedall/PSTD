# PQ55 — TCINFO bitmap boundary semantics

## Evidence from PQ54

The public PST fixture retained the proven 52-byte row width and reported:

- TCINFO region boundaries: `48:48:50:52`
- maximum column extent: `50`
- presence bitmap length: `2` bytes for 14 columns
- PQ54 bitmap end: `54`
- PQ54 layout status: `tc_row_layout_extents_out_of_bounds_52`

The overflow was produced by treating the final TCINFO boundary (`52`) as the start of the bitmap and then adding two bytes. That interpretation double-counted the bitmap. The same evidence supports a bounded trailing-bitmap interpretation: the final boundary is the end of the complete row layout, the bitmap occupies bytes `50..52`, and the maximum column extent ends at byte `50`.

## Revised requirement

PQ55 corrects only the structural boundary calculation:

1. preserve the existing four TCINFO boundaries;
2. preserve bitmap length as `ceil(column_count / 8)`;
3. treat the final TCINFO boundary as the bitmap end;
4. derive the bitmap start as `bitmap_end - bitmap_byte_len` when reporting or validating later;
5. retain the proven ordinal row-index interpretation and 52-byte row width;
6. do not read bitmap bits or column values.

## Expected public-fixture evidence

The fixture should continue to report:

- `row_width=52`
- `tcinfo_regions=48:48:50:52`
- `max_column_extent=50`
- `bitmap_bytes=2`
- `bitmap_end=52`
- `row_layout_status=tc_row_layout_extents_validated_52`
- `status=tc_subnode_rows_ordinal_index_validated_52`

Extraction totals must remain unchanged.

## Safety boundary

This change proves only that the structural regions fit inside the row. It does not establish bitmap bit ordering, nullability semantics, property type decoding, variable-value indirection, or row materialisation.

## PQ56 decision boundary

If the public fixture confirms the corrected layout, PQ56 should read only the two-byte presence bitmap for each of the four rows and report per-row set-bit counts. It must not decode column values. If the corrected layout still fails, the next run should expose the exact row-tail bytes and reassess the boundary interpretation before any row access.
