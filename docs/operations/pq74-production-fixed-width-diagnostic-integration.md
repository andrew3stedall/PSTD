# PQ74 — Production fixed-width diagnostic integration

## Objective

Connect the validated PQ72 fixed-width projection and PQ73 bounded diagnostic representation to the production table-heap reporting path without serialising row payload bytes or exposing partial property evidence.

## Repository finding

PQ73 was already merged before this run. No competing PQ pull request was open. The remaining gap was no longer another decoder abstraction: `report_table_heaps` already held the decoded payload set, TCINFO descriptors, subnode row-resolution report, bitmap masks, and fixed-data boundary needed to invoke the existing projection safely.

## Implementation

`TcHeapDiagnostic` now retains one `TcFixedWidthDiagnostic`.

For subnode-backed TC rows, `report_table_heaps`:

1. reuses the existing `TcSubnodeRowResolutionReport`;
2. invokes `project_fixed_width_row_evidence` with the same payload set, row NID, descriptors, bitmap masks, and fixed-data boundary;
3. converts the projection only through `build_fixed_width_diagnostic`;
4. appends the diagnostic's stable status fragment to the existing progress output.

For non-subnode rows and failed TC heap resolution, reporting emits an explicit unavailable diagnostic with no tag, offset, size, raw values, decoded values, or payload bytes.

## Safety properties

- No row payload bytes are serialised.
- Property metadata and values are published only when the PQ72 projection returns validated evidence.
- Failed and unavailable states remain empty and explicit.
- Existing row, bitmap, descriptor, layout, and extraction diagnostics are preserved.
- The implementation reuses the existing candidate resolver, transport bridge, selector, projection, and diagnostic builder rather than duplicating parsing logic.

## Extraction impact

This change does not claim a new semantic MAPI property and does not change message, body, or attachment counts. It makes the first validated fixed-width scalar evidence observable in the production fixture output. The exact public-fixture property tag and values must be recorded from the completed GitHub Actions run before selecting the next semantic milestone.

Current established baseline remains:

- TCINFO descriptors: 14
- rows: 4
- row width: 52 bytes
- row payload: 208 bytes
- validated offsets: `0,52,104,156`
- bitmap masks: `11111011000000` for each row
- messages extracted: 1
- body payloads: 2
- attachments: 0

## Revised PQ75 requirements

PQ75 must use the observed PQ74 fixture evidence rather than assuming a property meaning.

1. Record the selected property tag, type, offset, size, raw values, and decoded values from the public fixture.
2. Compare the tag against authoritative MAPI property definitions.
3. If the tag has a stable fixed-width semantic meaning, expose that one property end-to-end with a typed name and retain the raw evidence.
4. If the selected descriptor is not semantically useful, revise the selector to choose the highest-value supported fixed-width descriptor deterministically, with regression tests and no heuristic interpretation.
5. Do not begin variable-width string, body, recipient, or attachment work until the fixed-width production evidence is confirmed.
6. Preserve fail-closed behaviour and rerun the public fixture after the change.
