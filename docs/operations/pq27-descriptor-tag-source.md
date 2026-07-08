# PQ27 Descriptor Tag Source

## Goal

Capture the table descriptor tag-source evidence needed after PQ26 showed valid value extents with unknown tags.

## Scope

- Record first and second unknown table column tags in the parser report.
- Record low/high words for those unknown tags.
- Include message status lines in the public progress artifact.
- Surface PQ27 tag-source metrics in `progress_summary`.

## Boundary

PQ27 remains diagnostic-only. It does not materialize table values into message fields.
