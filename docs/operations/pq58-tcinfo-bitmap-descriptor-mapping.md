# PQ58 — TCINFO bitmap descriptor mapping validation

## Basis

PQ57 established four fixed-width 52-byte rows and exposed the exact bounded 14-bit masks from bytes `50..52`:

```text
11111011000000
11111011000000
11111011000000
11111011000000
```

Each row contains seven set and seven unset bits. That evidence does not by itself prove that TCINFO column descriptors form a valid one-to-one mapping onto bitmap positions.

## Revised scope

The earlier PQ58 proposal combined two separate claims: validating descriptor indices and associating row states with property tags. This run narrows PQ58 to the prerequisite parser invariant.

For a TCINFO containing `N` columns, the parser now requires every descriptor bitmap index to:

1. fall within `0..N`;
2. occur exactly once;
3. collectively cover the complete `0..N` range.

Descriptor order remains unchanged. A valid mapping may therefore have descriptors in a different order from their bitmap indices.

Duplicate or out-of-range indices cause the entire TCINFO parse to fail. No partial mapping is returned. Missing indices are checked explicitly as a defensive invariant, although with exactly `N` descriptors, uniqueness plus range already implies complete coverage.

## Safety boundary

PQ58 does not:

- call set bits present properties;
- decode fixed or variable column values;
- follow HNID/HID references stored in rows;
- reorder descriptors;
- change extraction totals.

## Tests

The focused tests cover:

- the normal complete mapping `0,1`;
- descriptor order independent from bitmap order using `1,0`;
- duplicate index rejection using `0,0`;
- out-of-range rejection using `0,2`;
- existing TCINFO bounds and HNID classification behaviour.

## Evidence required from CI

The branch must pass the complete repository workflow, including Rust tests, Clippy, rustfmt, Python, Docker, CLI smoke tests, and the public PST fixture. The fixture must retain the PQ57 row and mask evidence without reducing extraction output.

## Following run

PQ59 should publish the validated descriptor mapping as diagnostic evidence. For each bitmap index it should report the descriptor-order position, raw property tag, property type, data offset, data size, and the four raw row bit states. It must continue to avoid reading values or assigning semantic presence until that diagnostic mapping is confirmed against the fixture.
