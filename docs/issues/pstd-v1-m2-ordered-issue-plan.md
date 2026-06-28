# PSTD v1 M2 Ordered Issue Plan

## Execution rule

Work at milestone/epic level. Do not ask the user to prompt after each issue. Follow this order unless a blocking technical problem is discovered.

## Milestone

[M2: PST Binary Foundation](../milestones/pstd-v1-m2-pst-binary-foundation.md)

## Epic

[E2: PST Binary Reader and Structure Foundation](../epics/pstd-v1-e2-pst-binary-foundation.md)

## Ordered issue list

1. #19 / M2-I01: Add PST byte reader and bounded random-access API.
2. #20 / M2-I02: Parse and validate PST header metadata.
3. #21 / M2-I03: Define strongly typed PST primitive identifiers.
4. #22 / M2-I04: Add endian-aware binary parsing helpers.
5. #23 / M2-I05: Parse block and page trailer structures.
6. #24 / M2-I06: Implement BBT page parsing and block lookup skeleton.
7. #25 / M2-I07: Implement NBT page parsing and node lookup skeleton.
8. #26 / M2-I08: Add raw block loading and bounded block reassembly interface.
9. #27 / M2-I09: Wire `pstd inspect` to real PST structure inspection.
10. #28 / M2-I10: Add M2 diagnostics, fixture guidance, and handoff notes.

## Dependency order

```text
#19 -> #20 -> #21 -> #22 -> #23 -> #24 -> #25 -> #26 -> #27 -> #28
```

## Issue details

The full developer-ready issue bodies are stored in GitHub issues #19-#28. This file is the canonical milestone execution order.

## Scope guardrails

Do not include in M2:

- Folder tree extraction.
- Message metadata extraction.
- Body extraction.
- Attachment extraction.
- Address resolution.
- Snowflake loading.
- Web UI.
- Production deployment.
- Secrets, billing changes, production access, or destructive data behaviour.

## Required validation later

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
pstd --help
pstd inspect --help
pstd inspect --input <approved-small-fixture.pst>
```

## Fixture policy

Do not commit private PST files. Use tiny synthetic byte fixtures in tests, and approved small PST fixtures only in local or secure fixture storage.
