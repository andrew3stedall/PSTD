# PQ58 — TCINFO column-to-bitmap index mapping

## Evidence from PQ57

The public fixture has 14 TCINFO columns and four validated 52-byte rows. Each row exposes the same bounded mask, `11111011000000`, in least-significant-bit-first bitmap-index order. PQ57 proves that indices `0..13` can be read safely, but it does not prove which TCINFO descriptor owns each index or what a set bit means semantically.

## Revised requirement

PQ58 should build a bounded structural mapping from every parsed TCINFO column descriptor to its declared `bitmap_bit`:

1. retain the descriptor's raw property tag, property type (the low 16 bits of the tag), data offset, data size, and bitmap index;
2. validate that all bitmap indices are within `0..column_count`;
3. require the indices to be unique and to form the complete set `0..13` before pairing them with row masks;
4. for each validated row, report the raw `0` or `1` state associated with each descriptor;
5. preserve descriptor order and bitmap-index order separately so they cannot be conflated;
6. return no row-to-column mapping if any index is duplicated, missing, or out of range.

## Scope boundary

PQ58 must not read bytes from the data regions, follow variable-value references, materialise table properties, or claim that `1` means a semantically present value. It reports only the declared descriptor metadata and raw bitmap state.

## Required tests

- a shuffled descriptor order still maps through `bitmap_bit` correctly;
- duplicate, missing, and out-of-range bitmap indices are rejected without a partial mapping;
- 14-bit masks remain bounded and padding bits remain excluded;
- the reporting path preserves all 14 mapped descriptor records for each of four rows;
- existing PQ57 counts and masks remain unchanged.

## Required public-fixture evidence

The artifact must report:

- 14 parsed descriptors and 14 unique bitmap indices;
- complete index coverage `0..13`;
- four mapped rows;
- seven raw set and seven raw unset states per row;
- the raw property tag and property type paired with each index;
- unchanged extraction totals.

If the descriptor indices are not unique and complete, PQ59 must diagnose the exact duplicate/missing index set rather than attempting value decoding. If the mapping validates, PQ59 may define a separately bounded value-access experiment from the descriptor offsets and sizes.
