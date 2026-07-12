# PQ64 — bounded fixed-width row evidence selection

## Evidence basis

PQ63 published 14 validated TCINFO descriptor records for the public PST fixture and retained four validated 52-byte rows. Six four-byte descriptors are structurally eligible because their ranges fit below the bitmap region and their raw row bit state is `1111`.

The fixture evidence is not sufficient to interpret any candidate as a MAPI value. Property-tag representation and least-significant-bit-first bitmap ordering remain provisional. PQ64 therefore adds a bounded selection and raw-byte extraction primitive only.

## Implemented scope

`select_fixed_width_row_evidence` accepts:

- validated TCINFO column descriptors;
- complete binary bitmap masks;
- one resolved row payload;
- validated absolute row offsets;
- a validated fixed row width;
- the exclusive end of the fixed-data region.

A candidate must:

1. have a four-byte width;
2. fit completely inside the fixed-data region;
3. have a bitmap index within the descriptor count;
4. have a raw set bit in every supplied row;
5. fit inside every validated row and the row payload.

The selector returns raw lowercase hexadecimal bytes for every row. It prefers the candidate with the largest number of distinct row values, because that gives the strongest bounded evidence for the next run. Ties use the lowest bitmap index for deterministic output.

## Fail-closed behaviour

No partial evidence is returned when:

- masks and row offsets have different counts;
- masks have the wrong width or contain non-binary values;
- the fixed-data boundary exceeds the row width;
- row or column arithmetic overflows;
- any selected value exceeds the validated row or payload;
- no four-byte descriptor is set in every row.

## Safety boundary

PQ64 does not:

- interpret the raw bytes as an integer, timestamp, identifier, string reference, or MAPI value;
- follow HID or HNID references;
- treat a set bitmap bit as semantic property presence;
- change message, body, attachment, or output counts;
- publish fixture evidence through `TcHeapDiagnostic` yet.

## Validation requirements

Before squash merge:

- Rust build and all tests pass;
- Clippy passes with warnings denied;
- rustfmt passes;
- Python, Docker, CLI, and public-PST jobs remain green;
- extraction totals and PQ63 descriptor evidence remain unchanged.

## Proposed PQ65

Integrate the PQ64 selector with the resolved subnode-row reporting path. Compute validated row offsets from the existing direct or ordinal row mode, emit the selected descriptor metadata and four raw hexadecimal values, and record explicit `validated`, `unavailable`, or `construction_failed` status. Continue to avoid semantic interpretation.
