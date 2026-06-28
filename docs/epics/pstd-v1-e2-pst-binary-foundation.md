# Epic E2: PST Binary Reader and Structure Foundation

## Outcome

Create the first real PST binary reading foundation for `pstd` without extracting folders, messages, bodies, or attachments yet.

## Scope

This epic covers:

- Bounded byte reader for large PST files.
- Header parsing and validation.
- Strongly typed PST identifiers and references.
- Endian-aware parsing helpers.
- Block/page trailer parsing.
- Initial BBT and NBT lookup skeletons.
- Raw block loading interface.
- Real `pstd inspect` structural output.
- M2 diagnostics and deferred validation notes.

## Out of scope

- LTP/property context parsing.
- Table context parsing.
- Folder traversal.
- Message metadata extraction.
- Recipients/threading.
- Bodies and attachments.
- Snowflake.
- Web UI.
- Production deployment.

## System flow

```text
pstd inspect --input archive.pst
  -> config
  -> bounded byte reader
  -> header parser
  -> primitive IDs and refs
  -> trailer/page parsing
  -> BBT/NBT skeleton lookups
  -> structured inspect summary
```

## Success criteria

- `pstd inspect` no longer only prints a placeholder.
- Unsupported or invalid files produce structured errors.
- Large files are read by offset/range, not loaded completely into memory.
- Core PST structures are strongly typed enough for future parser work.
- M3 can build LTP/property/table context parsing on top of M2 without rewriting the reader layer.

## Risk notes

| Risk | Rating | Mitigation |
|---|---:|---|
| PST binary parsing errors create misleading later extraction results | High | Keep M2 focused on tested low-level structures |
| Real fixtures are sensitive | High | Use tiny approved or synthetic fixtures only |
| ANSI vs Unicode support ambiguity | Medium | Detect both; support Unicode first; report unsupported variants clearly |
| BBT/NBT implementation grows too large | Medium | Implement lookup skeletons and diagnostics first |
| Tests remain deferred | Medium | Document commands and fixture requirements clearly |

## Handoff to M3

M3 should consume the M2 reader/index APIs and add logical PST structures: LTP, heaps, property contexts, table contexts, folder traversal, and first message metadata records.
