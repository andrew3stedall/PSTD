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
| Table-context root parsing | PQ37 complete | A bounded 22-byte TCINFO parser and exact 8-byte TCOLDESC parser exist. HNID values are classified as null, HID, or NID without premature resolution. |
| Table row materialisation | In progress | The real `hidUserRoot`, `hidRowIndex`, `hnidRows`, and `hidIndex` references still need resolution through the correct address spaces. |
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
- unresolved BID `0x74`: 208 bytes.

PQ37 is deliberately output-neutral. It adds safe parser primitives required for the next reference-resolution step.

## Current active blocker

**PQ38: resolve and parse the real table-context root allocation.**

Required outcome:

1. Resolve the Heap-on-Node `hidUserRoot` allocation from the selected decoded payload.
2. Parse TCINFO and TCOLDESC records from that bounded allocation rather than from a guessed block prefix.
3. Preserve `hidRowIndex`, `hnidRows`, and `hidIndex` as typed references.
4. Emit deterministic fixture evidence before attempting row materialisation.
5. Do not assume BID `0x74` is the row matrix without reference-chain evidence.

## Active conversion coverage roadmap

1. **PQ38** — wire TCINFO parsing to the real `hidUserRoot` allocation and publish fixture evidence.
2. **PQ39** — resolve row-index and row-storage references through the correct HID/NID address spaces, revised from PQ38 evidence.
3. **PQ40+** — materialise rows and selected table properties only after structural bounds and reference ownership are proven.

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
