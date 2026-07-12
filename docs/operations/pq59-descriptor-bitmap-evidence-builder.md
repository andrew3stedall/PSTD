# PQ59 — Descriptor/bitmap evidence builder

## Evidence basis

PQ58 validated that TCINFO column descriptors form a complete one-to-one mapping over bitmap indices `0..N-1`. The public fixture retains four 14-bit masks (`11111011000000`) over four fixed 52-byte rows.

## Revised requirement

The earlier PQ59 proposal combined evidence construction with public diagnostic integration. That would mix a new mapping abstraction with changes to the extraction reporting surface. PQ59 is therefore limited to a reusable, bounded evidence builder.

For each validated descriptor it records:

- bitmap index;
- original descriptor-order position;
- raw property tag;
- property type derived from the upper 16 bits of the stored raw tag;
- data offset;
- data size;
- one raw `0`/`1` state per supplied row mask.

The result is sorted by bitmap index while preserving descriptor order as separate evidence.

## Safety boundary

The builder:

- requires every mask width to equal the TCINFO column count;
- rejects any non-binary mask character;
- relies on PQ58's parser invariant for unique, complete bitmap indices;
- does not read row value bytes;
- does not follow HID/HNID references;
- does not call a set bit a semantically present property.

## Validation required

The full GitHub Actions matrix must pass on the final branch head. Existing public-PST extraction totals and PQ57 mask evidence must remain stable.

## Following run

PQ60 should integrate this builder into the bounded table diagnostic and publish the 14 descriptor records for the public fixture. Only after that evidence is visible should a separately bounded fixed-width value-access experiment be considered.
