# Milestone M2: PST Binary Foundation

## Status

Planned. Not yet implemented.

## Goal

Implement the low-level PST binary foundation needed before folders, message metadata, bodies, and attachments can be extracted from real PST files.

M2 should make `pstd inspect --input <pst-file>` useful for confirming whether a file is a supported PST and for showing basic structure information.

## Included epic

- [Epic E2: PST binary reader and structure foundation](../epics/pstd-v1-e2-pst-binary-foundation.md)

## Included issues

1. M2-I01: Add PST byte reader and bounded random-access API.
2. M2-I02: Parse and validate PST header metadata.
3. M2-I03: Define strongly typed PST primitive identifiers.
4. M2-I04: Add endian-aware binary parsing helpers.
5. M2-I05: Parse block/page trailer structures.
6. M2-I06: Implement BBT page parsing and block lookup skeleton.
7. M2-I07: Implement NBT page parsing and node lookup skeleton.
8. M2-I08: Add raw block loading and bounded block reassembly interface.
9. M2-I09: Wire `pstd inspect` to real PST structure inspection.
10. M2-I10: Add M2 diagnostics, deferred tests, fixture guidance, and handoff notes.

## Dependency order

```text
I01 -> I02 -> I03 -> I04 -> I05 -> I06 -> I07 -> I08 -> I09 -> I10
```

## Completion criteria

M2 is complete when the repository has:

- Bounded random-access PST byte reader.
- Header parser with supported/unsupported variant detection.
- Typed PST identifiers for nodes, blocks, pages, references, and byte offsets.
- Binary parsing helpers.
- Block and page trailer parsing.
- Initial BBT lookup skeleton.
- Initial NBT lookup skeleton.
- Raw block loading interface.
- Real `pstd inspect --input <pst-file>` output using the binary foundation.
- Structured diagnostics using existing status/progress records.
- Deferred local testing notes.

## Out of scope

- Folder tree extraction.
- Message metadata extraction.
- Body extraction.
- Attachment extraction.
- Address resolution.
- Snowflake loading.
- Web UI.
- Production deployment.
- Secrets, billing changes, production access, or destructive data behaviour.

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
