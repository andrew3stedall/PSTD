# Vertical 01 — Classify the first observed fixed-width property

## Objective

Classify the first fixed-width property emitted by the production public-PST diagnostic so that table-internal bookkeeping is not mistaken for readable email metadata.

## Starting evidence

The merged production integration reported:

- property tag: `0x67f20003`
- property identifier: `0x67f2`
- property type: `0x0003` (`PT_LONG`)
- decoded values: `45`, `48`, `51`, `54`
- four validated rows at offsets `0`, `52`, `104`, and `156`

The values were structurally valid, but no semantic name had been assigned.

## Classification

Property identifier `0x67f2` is `PidTagLtpRowId`. It is an LTP/Table Context row identifier used for table bookkeeping. It is not a subject, sender, timestamp, recipient, body, attachment, or other user-readable message property.

Property identifier `0x67f3` is the related `PidTagLtpRowVer` table-internal property. Both are now explicitly classified as `TableInternal`.

The implementation intentionally leaves all other identifiers `Unknown` until their meaning is verified. Unknown properties remain eligible for later inspection; they are not assigned guessed names.

## Implementation

Added `src/pst/tc_property_classification.rs` with:

- constants for `PidTagLtpRowId` and `PidTagLtpRowVer`;
- a bounded `TcPropertyRole` classification;
- canonical names for the two verified LTP properties;
- `is_user_readable_candidate` to distinguish internal bookkeeping from potential message data;
- focused regression tests for the observed tag, the related row-version tag, and unknown-property behaviour.

The module is exported from `src/pst/mod.rs`.

## Extraction impact

This milestone does not increase message, body, recipient, attachment, EML, or output-byte counts. Its observable result is semantic correction: the first decoded scalar is now identified as internal row identity rather than progress toward a readable email field.

Validated extraction baseline remains:

- TCINFO descriptors: 14
- rows: 4
- row width: 52 bytes
- row payload: 208 bytes
- bitmap masks: `11111011000000` for each row
- messages extracted: 1
- body payloads: 2
- attachments: 0

## Safety properties

- No unknown property receives a guessed MAPI meaning.
- Property type and property identifier remain separate.
- Internal properties can be excluded from user-readable candidate selection without deleting their structural evidence.
- No payload bytes are serialized.

## Next vertical milestone

Update fixed-width candidate selection to rank verified table-internal properties below unknown or verified message properties, then run the public fixture and expose the next selected tag and values. The milestone is complete only if it produces a different observed property or proves that no non-internal supported fixed-width property is present.

After that evidence, choose either:

1. a verified useful fixed-width message property; or
2. the first variable-width/HID/HNID string path needed to recover a subject.
