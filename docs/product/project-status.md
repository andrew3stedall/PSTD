# PSTD Project Status

## Purpose

Provide a single current-state view of what PSTD can do, what has been validated, and what remains blocked.

## Current implementation state

| Area | Status | Notes |
|---|---|---|
| Rust CLI and structured output | M1-M25 complete and CI validated | `pstd inspect`, `extract`, `batch`, and `version` exist with TAR/JSONL output, run summaries, batch progress, resume support, Docker packaging, and operator docs. |
| Header, BBT, and NBT traversal | PQ1-PQ3 complete | Safe Unicode root selection and corrected B-tree page metadata/child-reference decoding produce 50 BBT and 63 NBT entries on the public fixture. |
| Folder and message discovery | PQ4-PQ5 complete | The public fixture produces 11 folders and one true message candidate instead of treating all NBT entries as messages. |
| Property and body coverage | PQ6-PQ10 complete | Coverage, selected/unknown property diagnostics, String8 support, tag-shape analysis, and Heap-on-Node/BTH parser foundations are present. |
| Payload and subnode discovery | PQ11-PQ35 complete | Bounded payload scans, message subnode selection, recursive Unicode SLBLOCK/SLENTRY decoding, target resolution, cycle guards, and evidence artifacts are implemented. |
| Payload decoding and structural admission | PQ36 complete | Non-internal `NDB_CRYPT_PERMUTE` blocks are decoded; internal blocks remain raw; Heap-on-Node clients are classified; invalid legacy table declarations are rejected. |
| Table-context and row-location parsing | PQ37-PQ50 complete | The real TC heap, row-index BTH, and NID-backed 208-byte row payload resolve through bounded address-space-specific paths. |
| Table row structure | PQ51-PQ57 complete | Four ordinal rows resolve at 52 bytes each; TCINFO extents fit; bitmap bytes `50..52` yield four exact 14-bit masks. No row values are decoded. |
| Table row materialisation | In progress | PQ58 must validate descriptor `bitmap_bit` uniqueness and coverage before any column values are accessed. |
| EML reconstruction | Not implemented | Current canonical output remains structured TAR + JSONL. Full RFC-compliant EML reconstruction is later work after extraction coverage stabilises. |
| Snowflake, UI, search, analytics | Parked | Downstream work remains out of scope until PST conversion coverage is reliable. |

## Merged parser-quality milestones

| Range | Result |
|---|---|
| PQ1-PQ5 | Correct root/index traversal and real folder/message candidate discovery. |
| PQ6-PQ10 | Property/body measurement, selected dictionary expansion, tag-shape diagnostics, and initial Heap-on-Node/BTH support. |
| PQ11-PQ16 | Payload boundary diagnosis, message subnode selection, decoding, interpretation, and classification. |
| PQ17-PQ23 | Table-probe counters, candidate measurement, row-matrix diagnostics, and property candidate discovery. |
| PQ24-PQ31 | Column/tag/descriptor experiments and propagation diagnostics. |
| PQ32-PQ35 | Invalid legacy descriptor assumption identified; raw payload captured as Unicode SLBLOCK; SLENTRY targets resolved recursively. |
| PQ36 | Correct payload decryption and structural admission produced the first material property/body extraction improvement. |
| PQ37 | Specification-aligned TCINFO root parser added without changing extraction output. |
| PQ38-PQ48 | Resolved TC heap allocations and row-index BTH structures, then wired bounded table-probe evidence into the extraction run. |
| PQ49-PQ57 | Resolved subnode row storage and validated row references, ordinal semantics, 52-byte row layout, bitmap boundaries, bit counts, and exact masks. |

## Latest validated fixture result

The checked-in public fixture remains:

- 50 BBT entries;
- 63 NBT entries;
- 11 folders;
- 1 true message candidate;
- 0 attachments currently emitted.

PQ36 corrected the property and body path:

- selected properties: **0 → 16**;
- unknown properties: **74 → 19**;
- text body recovered;
- RTF body recovered;
- fallback body rows: **1 → 0**;
- false table declarations rejected;
- the NID-backed row payload now resolves as 208 bytes and is validated as four 52-byte rows.

PQ57 remains extraction-output-neutral but proves the full bounded structural chain to four row masks. Each row reports `11111011000000`, with seven set and seven unset bits. The result does not yet establish semantic property presence.

## Current active blocker

**PQ58: validate the TCINFO column-to-bitmap index mapping.**

Required outcome:

1. Preserve raw TCINFO descriptor metadata including property tag, property type, offset, size, and `bitmap_bit`.
2. Prove the 14 bitmap indices are unique, in range, and complete over `0..13`.
3. Pair each descriptor with only the raw set/unset state from each validated mask.
4. Preserve descriptor order separately from bitmap-index order.
5. Do not decode values or claim semantic property presence.

## Active conversion coverage roadmap

1. **PQ58** — validate and expose the TCINFO descriptor-to-bitmap index mapping.
2. **PQ59** — if PQ58 validates, define a bounded descriptor-offset/value-access experiment; otherwise diagnose the exact mapping defect.
3. **PQ60+** — materialise selected table properties only after bitmap mapping, bounds, value type, and indirection rules are proven.

The exact requirements for each later PQ must be revised from the preceding CI artifact rather than fixed in advance.

## Validation expectations

Every parser-quality PR must pass:

- `cargo fmt --check`;
- `cargo clippy --all-targets --all-features -- -D warnings`;
- `cargo test --all`;
- Python wrapper smoke checks;
- Docker build;
- CLI fixture inspect/extract checks;
- deterministic public-PST artifact generation and review.

## Risk statement

The implementation has strong bounded parsing, diagnostics, and CI coverage, but extraction remains fixture-limited. A parser milestone is not evidence of general PST compatibility unless the public/sanitised fixture artifacts demonstrate the relevant structure and no regression in existing outputs.
