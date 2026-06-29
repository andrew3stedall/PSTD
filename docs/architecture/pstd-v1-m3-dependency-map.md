# PSTD v1 M3 Dependency Map

## Purpose

Order M3 so agents can implement metadata extraction at milestone/epic level without waiting for a user prompt after every issue.

## Dependency graph

```text
M3-I01 Node/block logical access
  -> M3-I02 Heap-on-node
  -> M3-I04 Property context
  -> M3-I05 Table context

M3-I02 Heap-on-node
  -> M3-I03 BTH
  -> M3-I04 Property context
  -> M3-I05 Table context

M3-I03 BTH
  -> M3-I04 Property context
  -> M3-I05 Table context

M3-I04 Property context
  -> M3-I06 MAPI registry
  -> M3-I08 Message metadata

M3-I05 Table context
  -> M3-I07 Folder hierarchy
  -> M3-I08 Message metadata

M3-I06 MAPI registry
  -> M3-I07 Folder hierarchy
  -> M3-I08 Message metadata

M3-I07 Folder hierarchy
  -> M3-I09 Archive output

M3-I08 Message metadata
  -> M3-I09 Archive output

M3-I09 Archive output
  -> M3-I10 CLI wiring
  -> M3-I11 Handoff docs
```

## Recommended execution order

1. M3-I01: Stabilise node and block access for logical PST layers.
2. M3-I02: Implement heap-on-node parsing foundation.
3. M3-I03: Implement BTH parsing foundation.
4. M3-I04: Implement property context parsing.
5. M3-I05: Implement table context parsing.
6. M3-I06: Add selected MAPI property registry and value decoding.
7. M3-I07: Traverse folder hierarchy and emit folder inventory.
8. M3-I08: Extract initial message metadata records.
9. M3-I09: Write folder and message metadata to structured archive outputs.
10. M3-I10: Wire metadata-only extraction into the CLI.
11. M3-I11: Add M3 diagnostics, tests, fixture guidance, and handoff notes.

## Parallelisable work

After I01-I03, these can proceed with limited overlap:

- I04 property context parser.
- I05 table context parser.
- I06 selected MAPI property registry.

## Stop conditions

Stop and report if implementation requires private PST fixtures committed to the repo, body extraction, attachment extraction, or changes to the v1 output contract that would break M1/M2 outputs.

## Handoff to M4

M4 should consume M3 folder and message metadata records to add recipients, threading fields, conversation fields, and address resolution.
