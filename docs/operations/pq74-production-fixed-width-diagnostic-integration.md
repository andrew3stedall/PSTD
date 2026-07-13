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

This change does not assign semantic MAPI meaning and does not change message, body, or attachment counts. It makes the first validated fixed-width scalar evidence observable in the production fixture output.

GitHub Actions run 508 produced the following bounded evidence from the public fixture:

- candidate status: `tc_row_payload_candidates_resolved`
- transport status: `tc_row_transport_validated`
- evidence status: `tc_fixed_width_evidence_validated`
- selected property tag: `0x67f20003`
- property identifier: `0x67f2`
- property type: `0x0003` (`PT_LONG`, signed 32-bit integer)
- data offset: `0`
- data size: `4`
- raw row values: `2d000000`, `30000000`, `33000000`, `36000000`
- decoded row values: `45`, `48`, `51`, `54`
- failure reason: `none`

The values are structurally validated but are not yet assigned a semantic property name. Property identifier `0x67f2` is in the PST/LTP internal-property range rather than a user-facing message field, so PQ75 must verify its specification meaning before exposing it as typed metadata.

Established extraction totals remain:

- TCINFO descriptors: 14
- rows: 4
- row width: 52 bytes
- row payload: 208 bytes
- validated offsets: `0,52,104,156`
- bitmap masks: `11111011000000` for each row
- messages extracted: 1
- body payloads: 2
- attachments: 0

No message-count, body, recipient, or attachment regression was observed.

## CI outcome

The first workflow attempt failed only at `cargo fmt --check`; Rust build, tests, Clippy, Python packaging, and Docker had passed. The rustfmt changes were applied to `src/pst/tc_reporting.rs` and GitHub Actions run 508 then passed Rust build, tests, Clippy, formatting, Python, Docker, CLI smoke checks, and the public PST fixture.

## Revised PQ75 requirements

PQ75 must use the observed PQ74 fixture evidence rather than assuming a property meaning.

1. Verify property identifier `0x67f2` against the authoritative PST/LTP specification and record its exact role.
2. Determine whether values `45`, `48`, `51`, and `54` are row identifiers, node identifiers, or another table-internal scalar.
3. If the property has stable extraction value, expose that one property end-to-end with a typed name while retaining the raw evidence.
4. If it is only table-internal bookkeeping, revise the selector to choose the highest-value supported fixed-width descriptor deterministically, with regression tests and no heuristic interpretation.
5. Do not begin variable-width string, body, recipient, or attachment work until this fixed-width evidence is classified; after classification, proceed directly to the smallest observable metadata slice or to variable-width/HID/HNID resolution.
6. Preserve fail-closed behaviour and rerun the public fixture after the change.
