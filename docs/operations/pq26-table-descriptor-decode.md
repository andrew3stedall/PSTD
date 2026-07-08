# PQ26 Table Descriptor Decode

## Goal

Expose descriptor-level evidence for the table column/value layout after PQ25 found no byte-swapped or word-order tag signal.

## Scope

- Surface descriptor column count.
- Surface descriptor row count.
- Surface valid value extents.
- Surface omitted value extents.
- Surface unknown value extents.
- Decide whether the next blocker is tag-source decode or offset/width decode.

## Boundary

PQ26 remains diagnostic-only. It does not materialize table values into message fields.
