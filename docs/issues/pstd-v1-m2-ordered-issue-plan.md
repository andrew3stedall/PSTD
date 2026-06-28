# PSTD v1 M2 Ordered Issue Plan

## Execution rule

Work at milestone/epic level. Do not ask the user to prompt after each issue. Follow this order unless a blocking technical problem is discovered.

## Milestone

[M2: PST Binary Foundation](../milestones/pstd-v1-m2-pst-binary-foundation.md)

## Epic

[E2: PST Binary Reader and Structure Foundation](../epics/pstd-v1-e2-pst-binary-foundation.md)

---

## M2-I01: Add PST byte reader and bounded random-access API

### Goal

Create a safe byte reader for large PST files that supports bounded reads by offset without loading the full file into memory.

### In scope

- `PstByteReader` or equivalent.
- `read_at(offset, len)` API.
- File size discovery.
- Bounds checking.
- Structured errors with offset and requested length.
- Unit tests using tiny byte fixtures.

### Out of scope

- Real PST structure parsing.
- Memory-mapped implementation if it complicates portability.
- Folder/message extraction.

### Acceptance criteria

- Reads arbitrary byte ranges from a file.
- Rejects out-of-bounds reads with structured errors.
- Does not read the whole file into memory.
- Exposes file size.
- Existing placeholder reader boundary is replaced or adapted.

### Dependencies

None.

### Tests

Run later:

```text
cargo test --all
```

---

## M2-I02: Parse and validate PST header metadata

### Goal

Read enough PST header metadata to determine whether the input is a PST and whether the variant is supported.

### In scope

- Signature validation.
- Basic PST version/variant detection.
- Unicode vs ANSI detection where possible.
- Root pointers needed by later BBT/NBT work.
- `PstHeaderSummary` populated with real fields.
- Structured unsupported-file diagnostics.

### Out of scope

- Full BBT/NBT traversal.
- Message extraction.

### Acceptance criteria

- Valid-looking PST headers produce a header summary.
- Non-PST files fail cleanly.
- Unsupported variants are identified without panic.
- `pstd inspect` can show header-level information once wired.

### Dependencies

M2-I01.

---

## M2-I03: Define strongly typed PST primitive identifiers

### Goal

Replace raw integers with meaningful strongly typed identifiers used by the parser layers.

### In scope

- Node ID type.
- Block ID type.
- Byte offset type.
- Page reference type.
- Block reference type.
- Root pointer type.
- Debug formatting useful for diagnostics.

### Out of scope

- Full semantic validation of every identifier type.

### Acceptance criteria

- Core parser APIs do not pass raw `u64`/`usize` everywhere.
- Identifier types are Copy/Clone/Debug/Eq where useful.
- Conversions are explicit.
- Tests cover formatting and construction.

### Dependencies

M2-I01, can proceed alongside M2-I02.

---

## M2-I04: Add endian-aware binary parsing helpers

### Goal

Centralise low-level little-endian parsing and slicing helpers.

### In scope

- Helpers for `u8`, `u16`, `u32`, `u64`, signed values where required.
- Fixed-length byte slicing.
- Error on short buffers.
- Offset-aware diagnostics.

### Out of scope

- External binary parser frameworks unless justified.

### Acceptance criteria

- Parser code uses shared helpers rather than ad hoc byte indexing.
- Short buffers produce structured errors.
- Tests cover all helper functions.

### Dependencies

M2-I01.

---

## M2-I05: Parse block/page trailer structures

### Goal

Parse reusable low-level trailer structures needed for PST pages and blocks.

### In scope

- Page trailer parser.
- Block trailer parser.
- Basic CRC/signature fields where practical.
- Diagnostics for mismatched sizes or unsupported variants.

### Out of scope

- Full CRC validation if it delays M2.
- Full block reassembly.

### Acceptance criteria

- Trailer structs parse from byte slices.
- Invalid lengths fail cleanly.
- Parsed values are printable in diagnostics.

### Dependencies

M2-I03, M2-I04.

---

## M2-I06: Implement BBT page parsing and block lookup skeleton

### Goal

Parse Block B-tree pages enough to support future block lookups.

### In scope

- BBT page structure definitions.
- BBT entry parsing.
- Root page loading from header data.
- Basic traversal skeleton.
- Lookup interface by block ID.
- Clear `not_implemented` diagnostics for unsupported page forms.

### Out of scope

- Full corruption recovery.
- Deep optimisation.

### Acceptance criteria

- BBT pages can be parsed from controlled fixtures.
- Lookup API exists even if some variants return structured unsupported errors.
- No panics on malformed pages.

### Dependencies

M2-I02, M2-I03, M2-I04, M2-I05.

---

## M2-I07: Implement NBT page parsing and node lookup skeleton

### Goal

Parse Node B-tree pages enough to support future node lookup.

### In scope

- NBT page structure definitions.
- NBT entry parsing.
- Root page loading from header data.
- Basic traversal skeleton.
- Lookup interface by node ID.
- Clear diagnostics for unsupported page forms.

### Out of scope

- LTP/property parsing.
- Folder/message enumeration.

### Acceptance criteria

- NBT pages can be parsed from controlled fixtures.
- Node lookup API exists.
- Malformed pages return structured errors.

### Dependencies

M2-I02, M2-I03, M2-I04, M2-I05.

---

## M2-I08: Add raw block loading and bounded block reassembly interface

### Goal

Expose a safe interface for loading raw PST block data from BBT-derived locations.

### In scope

- Load raw block by block ID/lookup result.
- Bounds check offset and size.
- Return owned bytes or bounded slices according to implementation design.
- Define future multi-block reassembly interface.
- Diagnostics for missing or unsupported block forms.

### Out of scope

- LTP decoding.
- Attachment/body extraction.

### Acceptance criteria

- Raw block loader has tests with controlled byte fixtures.
- Out-of-bounds block metadata fails safely.
- Later LTP work can consume this API.

### Dependencies

M2-I06.

---

## M2-I09: Wire `pstd inspect` to real PST structure inspection

### Goal

Replace the M1 placeholder `inspect` command with real header and structure summary output.

### In scope

- Inspect uses byte reader.
- Inspect parses header.
- Inspect attempts BBT/NBT root parsing where available.
- Human-readable output.
- Optional JSON output flag if simple.
- Non-PST and unsupported PST diagnostics.

### Out of scope

- Folder/message listing.
- Full validation report.

### Acceptance criteria

- `pstd inspect --input <file>` performs real file reads.
- Invalid files produce non-zero exit with useful error.
- Valid fixtures produce basic structural summary.

### Dependencies

M2-I01 through M2-I08.

---

## M2-I10: Add M2 diagnostics, deferred tests, fixture guidance, and handoff notes

### Goal

Finish M2 with clear docs and validation expectations.

### In scope

- Update milestone checklist.
- Update deferred testing plan for PST reader/parser tests.
- Add fixture guidance.
- Update roadmap/changelog.
- Record known limitations.

### Out of scope

- Creating or committing private PST fixtures.

### Acceptance criteria

- Docs clearly state what M2 implements.
- Docs clearly state what is still deferred to M3.
- Tests not run are explicitly listed.
- Fixture policy warns against committing private PSTs.

### Dependencies

All previous M2 issues.
