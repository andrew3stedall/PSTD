# PQ51 — validate fixed row width

## Evidence from PQ50

The public PST fixture resolved one table row matrix through NID `0x809f` to one 208-byte payload. Four row references were present and all four were within the payload. Existing extraction totals remained stable at 11 folders, one extracted message, two body payloads, zero attachments, one tar shard, and 29,806 output bytes.

## Revised requirement

The previous PQ51 proposal combined fixed-row-width inference with TCINFO column-boundary validation. Those are separate proofs at different layers. The subnode row resolver has the row references and row payload length, but not the TCINFO column descriptors. PQ51 therefore validates only whether the references partition the payload into equal, non-zero rows beginning at offset zero.

Column-region and descriptor extent checks remain the next step only if the real fixture confirms a fixed-width layout.

## Changes

- Derive row widths from consecutive row references and the final payload tail.
- Require references to be strictly increasing.
- Require the first row to start at offset zero.
- Require every derived row width to be equal and non-zero.
- Preserve bounds checking before width inference.
- Report the inferred width through the bounded diagnostic status.
- Reject variable, duplicate, descending, or non-zero-start layouts without reading row values.

## Before versus after

| Measure | Before PQ51 | PQ51 target |
|---|---:|---:|
| Row payload bytes | 208 | 208 |
| Row references | 4 | 4 |
| References in bounds | 4 | 4 |
| Fixed-width proof | Missing | Explicit |
| Expected inferred width | Unproven | 52 bytes if fixture evidence agrees |
| Column values decoded | 0 | 0 |
| Extraction totals | Existing baseline | Unchanged |

## Safety boundary

PQ51 does not interpret property values, presence bitmaps, or TCINFO column extents. A fixed-width result proves only the partition of the row payload. It does not prove that any column offset or value type is safe to decode.

## CI history

GitHub Actions run #426 passed Rust build, Rust tests, Clippy, Python, and Docker. It failed only at `cargo fmt --check`, which required a mechanical layout change around the fixed-width inference conditional in `src/pst/tc_subnode_rows.rs`. The exact formatter output was committed without changing logic or test expectations. The CLI and public-PST job was skipped because it depends on the Rust job.

The replacement run must validate the final branch head and produce the public fixture evidence before merge.

## Evidence required from CI

The public fixture should retain the PQ50 extraction totals and replace `tc_subnode_rows_references_validated` with either:

- `tc_subnode_rows_fixed_width_validated_52`; or
- `tc_subnode_rows_variable_or_invalid_width`.

Either result is valid evidence. CI must pass Rust build, tests, Clippy, rustfmt, Python, Docker, CLI smoke tests, and the public PST progress fixture before merge.

## Proposed PQ52 decision boundary

- If the fixture validates 52-byte fixed rows, carry TCINFO data-region boundaries and maximum column extents into the diagnostic and verify they fit within 52 bytes.
- If the fixture reports variable or invalid width, inspect the exact row references and determine whether they are offsets, row indices, or require another size source.
- Do not decode presence bitmaps or property values until both row partitioning and TCINFO extents are proven safe.
