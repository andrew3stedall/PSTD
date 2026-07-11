# PQ53 — validate row-index ordinal semantics

## Evidence from PQ52

GitHub Actions run #430 passed the full CI matrix. The public PST fixture preserved 11 folders, one extracted message, two body payloads, zero attachments, one tar shard, and 29,806 output bytes.

The table probe exposed one 208-byte row payload with four row-index values:

- `row_reference_values=0:1:2:3`
- `row_spans=1:1:1:205`

Those values disprove the direct-byte-offset interpretation used by PQ51. They are a contiguous zero-based sequence and are therefore more consistent with row ordinals than byte positions.

## Revised requirement

PQ53 validates a bounded ordinal interpretation without reading row contents:

1. Preserve the existing direct-offset fixed-width check for fixtures that genuinely provide offsets.
2. When direct offsets do not form equal rows, require row references to equal `0..row_count-1` exactly.
3. Require the resolved row payload length to divide evenly by the row count.
4. Infer row width as `payload_bytes / row_count` only when both conditions hold.
5. Reject missing, duplicated, non-contiguous, reordered, or non-divisible ordinal layouts.
6. Continue to avoid presence-bitmap, column, and property-value decoding.

## Expected public-fixture evidence

For the current fixture, 208 bytes divided by four contiguous ordinal references should produce a validated 52-byte row width and status:

`tc_subnode_rows_ordinal_index_validated_52`

Extraction totals must remain unchanged.

## CI evidence

GitHub Actions run #432 compiled successfully, passed the Rust tests, Python wrapper, and Docker build, but failed the Rust job at Clippy before rustfmt and the CLI/public-PST fixture could run. Clippy identified the manual remainder expression used for the divisibility guard and required `usize::is_multiple_of` under the repository's `-D warnings` policy.

The implementation was updated mechanically to use `!row_data_byte_len.is_multiple_of(row_references.len())`. No row-index logic, acceptance criteria, test expectations, or extraction behavior changed. The replacement CI run must pass the complete matrix and publish the public-fixture result before merge.

## Safety boundary

This PQ proves only the row-reference mode and fixed partition size. It does not prove that TCINFO column offsets fit within 52 bytes or that row bytes can be interpreted safely.

## Proposed PQ54

If CI confirms the 52-byte ordinal interpretation, validate TCINFO data-region boundaries, maximum column extents, and bitmap offsets against that row width before decoding any value. If the fixture does not confirm it, inspect row-index key/value semantics again rather than adding another heuristic.
