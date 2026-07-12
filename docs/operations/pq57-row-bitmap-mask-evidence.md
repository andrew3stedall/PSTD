# PQ57: bounded row bitmap mask evidence

## Evidence entering this run

PQ56 validated one uniquely resolved 208-byte row payload containing four zero-based ordinal rows of 52 bytes. TCINFO places the bounded bitmap at bytes `50..52` of each row and declares 14 columns. The public fixture reported four rows with seven set and seven unset bits each, but counts alone could not show whether the masks were identical or how individual bit positions varied.

## Revised requirement

PQ57 exposes the exact first 14 bits from each validated bitmap, in TCINFO bitmap-index order. Bit index zero is rendered first and each byte is traversed least-significant bit first, matching the bounded count implementation. The two unused padding bits in the second byte are excluded.

The implementation retains the existing count evidence as a consistency check and records one 14-character mask per analyzed row. It does not call set bits present properties, associate them with property tags, or read any column-value bytes.

## Safety boundary

Mask extraction is attempted only after unique payload resolution, fixed-width row validation, valid direct-offset or ordinal row mode, sufficient bitmap capacity, and per-row bounds checks. Any failed precondition returns no masks.

## CI and public-fixture evidence

GitHub Actions run #453 passed on head `404f395dfe46e2d10eedb283567578f5b8cf5fca`, including Rust build and tests, Clippy, rustfmt, Python, Docker, CLI smoke tests, and the public PST fixture.

The fixture preserved the validated structure and exposed four identical bounded masks:

- rows: `4`;
- row width: `52` bytes;
- bitmap range: `50..52`;
- set counts: `7:7:7:7`;
- unset counts: `7:7:7:7`;
- masks: `11111011000000:11111011000000:11111011000000:11111011000000`;
- bitmap status: `tc_row_bitmap_masks_validated`;
- row-layout status: `tc_row_layout_extents_validated_52`;
- row-index status: `tc_subnode_rows_ordinal_index_validated_52`.

Extraction totals remained stable at 11 folders, one extracted message, two body payloads, zero attachments, one tar shard, and 29,806 output bytes.

## Interpretation

The four rows carry identical raw 14-bit masks. This is stronger evidence than the earlier count-only result, but it still does not prove semantic property presence or independently validate least-significant-bit-first ordering. The result should be treated as bounded structural evidence only.

## PQ58 decision boundary

PQ58 should pair each mask position with the TCINFO column descriptor carrying that bitmap index and report only:

- bitmap index;
- property tag;
- property type;
- raw set or unset state for each row.

PQ58 must preserve descriptor order and verify that all bitmap indices are unique and within the 14-column range. It must not decode column values, follow variable-value references, or call set bits present properties until the index-to-column relationship is independently validated.

## Reporting completion

The row-resolution report carries the exact masks into `TcHeapDiagnostic`, whose bounded status fragment renders them as `bitmap_masks=...`. A reporting-level regression test verifies that four masks survive the full in-memory reporting path instead of being dropped after row analysis.
