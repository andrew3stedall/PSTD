# PQ56 — bounded row bitmap counts

## Evidence from PQ55

The complete CI workflow passed after correcting the final TCINFO boundary semantics. The public fixture is expected to retain a 52-byte row width, `tcinfo_regions=48:48:50:52`, a two-byte trailing bitmap, `bitmap_end=52`, and `tc_row_layout_extents_validated_52`.

## Revised requirement

PQ56 reads only bytes `50..52` from each validated 52-byte row and counts set and unset bits across the 14 declared columns. It deliberately reports bit counts rather than claiming that a set bit means a semantically present property. Bit ordering and property-value decoding remain outside scope.

The implementation must:

1. require a uniquely resolved row payload and validated fixed-width row mode;
2. derive row byte offsets from direct offsets or zero-based ordinals as already proven;
3. require the bitmap range to fit inside every row;
4. require enough bitmap capacity for all columns;
5. count only the first 14 bits, excluding padding bits;
6. publish per-row set and unset counts plus a bounded bitmap status;
7. avoid reading any column-value bytes.

## Implementation evidence

The branch implementation workflow completed its targeted formatting and Rust tests successfully, committed the permanent Rust changes, and removed both temporary workflow files. The final branch head must still pass the repository's complete CI and public-PST fixture before merge.

## Expected public-fixture evidence

The fixture should report four analyzed rows, four set-count values, four unset-count values, and `bitmap_status=tc_row_bitmap_counts_validated`. Extraction totals should remain unchanged.

## PQ57 decision boundary

If the counts are stable and structurally valid, PQ57 should expose the exact 14-bit masks per row and compare them with the TCINFO column bitmap indices. It still should not decode property values. If bitmap access fails, PQ57 should inspect the exact final two bytes of each row without assigning semantics.
