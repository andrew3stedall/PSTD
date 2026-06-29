# Milestone M2: PST Binary Foundation

## Status

Implemented and merged to `main` via PR #30.

Local validation remains deferred. Do not treat M2 as release-verified until the documented validation commands have run from Codex/laptop or CI.

## Goal

Implement the low-level PST binary foundation needed before folders, message metadata, bodies, and attachments can be extracted from real PST files.

M2 makes `pstd inspect --input <pst-file>` perform real file reads, parse PST header metadata, and attempt low-level index skeleton parsing when root offsets are available.

## Included epic

- [Epic E2: PST binary reader and structure foundation](../epics/pstd-v1-e2-pst-binary-foundation.md)

## Completed issues

1. #19 / M2-I01: Add PST byte reader and bounded random-access API.
2. #20 / M2-I02: Parse and validate PST header metadata.
3. #21 / M2-I03: Define strongly typed PST primitive identifiers.
4. #22 / M2-I04: Add endian-aware binary parsing helpers.
5. #23 / M2-I05: Parse block/page trailer structures.
6. #24 / M2-I06: Implement BBT page parsing and block lookup skeleton.
7. #25 / M2-I07: Implement NBT page parsing and node lookup skeleton.
8. #26 / M2-I08: Add block loading and bounded block interface.
9. #27 / M2-I09: Wire `pstd inspect` to real PST structure inspection.
10. #28 / M2-I10: Add M2 diagnostics, deferred tests, fixture guidance, and handoff notes.

## Implemented foundation

- Bounded range reader.
- PST header parser with version and variant summary.
- Strongly typed PST identifiers and references.
- Endian-aware binary parsing helpers.
- Page and block trailer parsers.
- BBT page and index skeleton.
- NBT page and index skeleton.
- Bounded block loading interface.
- `pstd inspect --input <file>` human-readable and JSON output.
- Unit tests using synthetic byte fixtures.

## Out of scope retained

- Folder tree extraction.
- Message metadata extraction.
- Body extraction.
- Attachment extraction.
- Address resolution.
- Snowflake loading.
- Web UI.
- Production deployment.

## Validation expectations

Run later from Codex/laptop or CI:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
pstd --help
pstd inspect --help
pstd inspect --input <approved-small-fixture.pst>
```

Do not commit private PST files. Use tiny synthetic or approved fixtures only.

## Next milestone

M3 should build on M2 by implementing logical PST structures, folder traversal, folder inventory, and initial message metadata extraction.
