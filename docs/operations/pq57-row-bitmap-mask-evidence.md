# PQ57: bounded row bitmap mask evidence

## Evidence entering this run

PQ56 validated one uniquely resolved 208-byte row payload containing four zero-based ordinal rows of 52 bytes. TCINFO places the bounded bitmap at bytes `50..52` of each row and declares 14 columns. The public fixture reported four rows with seven set and seven unset bits each, but counts alone cannot show whether the masks are identical or how individual bit positions vary.

## Revised requirement

PQ57 exposes the exact first 14 bits from each validated bitmap, in TCINFO bitmap-index order. Bit index zero is rendered first and each byte is traversed least-significant bit first, matching the bounded count implementation. The two unused padding bits in the second byte are excluded.

The implementation retains the existing count evidence as a consistency check and records one 14-character mask per analyzed row. It does not call set bits present properties, associate them with property tags, or read any column-value bytes.

## Safety boundary

Mask extraction is attempted only after unique payload resolution, fixed-width row validation, valid direct-offset or ordinal row mode, sufficient bitmap capacity, and per-row bounds checks. Any failed precondition returns no masks.

## Required CI and fixture evidence

The complete Rust, Python, Docker, CLI, and public-PST workflow must pass. The fixture must preserve:

- four rows and a 52-byte row width;
- bitmap range `50..52`;
- four analyzed rows;
- set counts `7:7:7:7` and unset counts `7:7:7:7`;
- four exact 14-character masks;
- unchanged extraction totals.

## PQ58 decision boundary

If four stable masks are exposed, PQ58 should pair each mask position with the TCINFO column descriptor carrying that bitmap index and report only raw set/unset state, property tag, and property type. It must not decode values or assert semantic presence until the index mapping is independently validated.

If masks cannot be produced, the following run should expose the exact trailing two bytes per row and reassess bit ordering and bitmap boundaries.

## Reporting completion

The row-resolution report carries the exact masks into `TcHeapDiagnostic`, whose bounded status fragment renders them as `bitmap_masks=...`. A reporting-level regression test verifies that four masks survive the full in-memory reporting path instead of being dropped after row analysis.
