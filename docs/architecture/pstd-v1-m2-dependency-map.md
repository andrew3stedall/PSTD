# PSTD v1 M2 Dependency Map

## Purpose

Order M2 so agents can implement PST binary foundation work at milestone/epic level without waiting for a user prompt after every issue.

## Dependency graph

```text
M2-I01 Byte reader
  -> M2-I02 Header parser
  -> M2-I04 Binary helpers

M2-I03 Primitive identifiers
  -> M2-I05 Block/page trailer parsing
  -> M2-I06 BBT parser
  -> M2-I07 NBT parser
  -> M2-I08 Raw block loading

M2-I02 Header parser
  -> M2-I06 BBT parser
  -> M2-I07 NBT parser
  -> M2-I09 Inspect command

M2-I04 Binary helpers
  -> M2-I05 Trailer parsing
  -> M2-I06 BBT parser
  -> M2-I07 NBT parser

M2-I05 Trailer parsing
  -> M2-I06 BBT parser
  -> M2-I07 NBT parser

M2-I06 BBT parser
  -> M2-I08 Raw block loading
  -> M2-I09 Inspect command

M2-I07 NBT parser
  -> M2-I09 Inspect command

M2-I09 Inspect command
  -> M2-I10 Handoff docs
```

## Recommended execution order

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

## Parallelisable work

After I01-I04, these can proceed with limited overlap:

- I05 trailer parsing.
- I06 BBT parser.
- I07 NBT parser.

## Stop conditions

Stop and report if implementation requires:

- Real private PST files committed to the repo.
- Third-party PST parser dependency.
- Folder/message/body/attachment extraction.
- Snowflake, frontend, deployment, production access, or secrets.

## Handoff to M3

M3 should consume the reader, header, BBT/NBT, and raw block APIs. M3 should not rewrite the byte reader unless M2 validation proves it incorrect.
