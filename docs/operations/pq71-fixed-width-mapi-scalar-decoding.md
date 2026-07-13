# PQ71 — bounded fixed-width MAPI scalar decoding

## Repository finding

PQ64 could select bounded four-byte row evidence, and PQ65-PQ70 established validated row offsets, payload transport, candidate resolution, and metadata projection. However, the selected evidence still exposed only hexadecimal bytes. The largest immediate capability gap was therefore fixed-width MAPI interpretation rather than another transport abstraction.

The previously proposed PQ71 reporting-only metadata integration was deferred. That integration would improve diagnostics but would not decode a new property value. This milestone instead extends the already validated fixed-width evidence boundary with strict scalar interpretation.

## Implemented contract

`select_fixed_width_row_evidence` now accepts only supported property-type and descriptor-size pairs and returns both raw hexadecimal bytes and decoded scalar strings.

Supported types:

| MAPI type | Code | Required size | Decoding |
|---|---:|---:|---|
| `PT_I2` | `0x0002` | 2 bytes | signed little-endian 16-bit integer |
| `PT_LONG` | `0x0003` | 4 bytes | signed little-endian 32-bit integer |
| `PT_BOOLEAN` | `0x000b` | 2 bytes | canonical `0`/`1` only |
| `PT_I8` | `0x0014` | 8 bytes | signed little-endian 64-bit integer |

Unsupported types and size mismatches are excluded rather than guessed. A noncanonical Boolean value fails the entire evidence construction without returning partial decoded output.

## Architectural decisions

- Reused the existing bounded descriptor selection, bitmap validation, row-offset validation, and row-bound checks.
- Preserved raw hexadecimal evidence alongside decoded values for auditability.
- Used the property-type portion of the MAPI property tag as the decoding discriminator.
- Kept variable-width values, floating-point values, time values, currency, errors, HID/HNID indirection, and semantic property naming out of scope.
- Continued deterministic candidate selection using decoded distinct-value count and lowest bitmap-index tie-breaking.

## Regression coverage

Tests cover:

- signed `PT_I2`, `PT_LONG`, and `PT_I8` decoding;
- canonical false and true `PT_BOOLEAN` values;
- rejection of noncanonical Boolean values;
- unsupported type exclusion;
- size-aware selection;
- deterministic tie-breaking;
- bitmap-state rejection;
- row-bound rejection with no partial evidence.

## Extraction evidence

This milestone adds a validated fixed-width decoding capability to the parser library. It does not yet alter the public fixture extraction totals because the production `report_table_heaps` call site does not publish selected fixed-width evidence.

Validated baseline remains:

| Measure | Baseline |
|---|---:|
| TCINFO descriptors | 14 |
| Rows | 4 |
| Row width | 52 bytes |
| Row payload | 208 bytes |
| Absolute offsets | `0,52,104,156` |
| Bitmap masks | `11111011000000` × 4 |
| Messages extracted | 1 |
| Body payloads | 2 |
| Attachments | 0 |

## Safety boundary

PQ71 does not:

- select ambiguous row payload candidates;
- bypass bitmap or row-bound validation;
- decode unsupported fixed-width types;
- treat arbitrary nonzero values as Boolean true;
- decode variable-width or HID/HNID-backed values;
- publish decoded values in extraction output;
- change message, body, recipient, or attachment counts.

## Progress estimate

Percentages reflect implemented, validated functionality rather than remaining effort.

| Area | Estimate | Basis |
|---|---:|---|
| Parser infrastructure | 88% | BBT/NBT, blocks, subnodes, HN, BTH, TCINFO, row-index, row payload and bounded transport exist. |
| Property decoding | 42% | Existing selected property support plus strict scalar decoding for four common fixed-width MAPI types; variable-width and indirection remain incomplete. |
| MAPI interpretation | 30% | Property tags and selected scalar types are understood, but broad type and semantic coverage is absent. |
| Message metadata | 35% | One fixture message and selected properties are emitted; table-backed metadata is not materialised. |
| Body extraction | 45% | Text and RTF payloads are recovered on the fixture; HTML/RTF normalization and broad compatibility remain incomplete. |
| Recipient extraction | 12% | Output foundations exist but recipient table materialisation is not implemented. |
| Attachment extraction | 8% | Output foundations exist; attachment table and payload extraction are not implemented. |
| Overall email extraction capability | 32% | One fixture message and body content extract, but reliable table-property, recipient, attachment, and EML reconstruction coverage is still missing. |

## Proposed PQ72

Integrate validated row transport and fixed-width scalar evidence into `report_table_heaps` as one fail-closed production slice:

1. resolve exactly one row payload through the PQ69/PQ68 path;
2. reuse validated absolute offsets and row width;
3. invoke `select_fixed_width_row_evidence` with TCINFO descriptors and masks;
4. publish raw and decoded values only when every boundary validates;
5. record public-fixture property tag, type, raw bytes, and decoded values;
6. preserve all existing extraction totals and diagnostics unless evidence demonstrates a safe materialisation path.
