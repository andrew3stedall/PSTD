# PSTD System Overview

_Last reviewed: 14 July 2026._

## Purpose

Describe the implemented PSTD architecture, current extraction flow, validated boundaries, and the remaining path to reliable email reconstruction.

## System direction

PSTD is a Rust-first PST parser and extraction engine. It reads PST structures directly, produces deterministic structured TAR/JSONL output, and records explicit partial/unsupported states. Python is an operator wrapper. Snowflake, UI, search, analytics, and graph systems are future consumers and do not parse PST internals.

## Runtime flow

```text
CLI / Python wrapper
  -> validated runtime configuration
  -> bounded PST reader
  -> header and root selection
  -> BBT/NBT traversal
  -> block, node, and subnode loading
  -> Heap-on-Node / BTH / Property Context / Table Context parsing
  -> folder, message, property, body, recipient, threading, and attachment projections
  -> extraction engine
  -> TAR shards + JSONL records + summaries + progress diagnostics
```

## Parsing layers

| Layer | Responsibility | Current maturity |
|---|---|---|
| Bounded I/O | Range-checked file reads and checked arithmetic. | Implemented |
| Header/root | PST variant metadata and safe root candidate selection. | Implemented and fixture validated |
| NDB traversal | BBT/NBT pages, block lookup, logical nodes, subnodes, trailers, and payload loading. | Implemented with guards; coverage remains layout-dependent |
| Heap-on-Node and BTH | Heap allocation lookup and B-tree-on-heap maps. | Implemented and reused by property/table paths |
| Property Context | Selected MAPI property lookup and decoding. | Implemented for the validated subset |
| Table Context | TCINFO, descriptors, row-index BTH, row payload resolution, row addressing, bitmap evidence, and value projection. | Implemented through bounded fixed-width and recipient string paths |
| Semantic extraction | Folder/message classification, bodies, recipient roles/names/addresses, threading, attachment evidence. | Partial but materially functional |
| Reporting | Bounded diagnostics, progress artifacts, structured output records. | Implemented; some validated diagnostics are not yet wired to final records |

## Current Table Context recipient flow

```text
validated TC heap
  -> TCINFO descriptors
  -> row-index BTH
  -> subnode-backed row payload
  -> four validated 52-byte rows
  -> property descriptor selection
  -> fixed-width recipient role values
  -> variable-width recipient name/address HNIDs
  -> Heap-on-Node string resolution
  -> authoritative property diagnostics
  -> row-aligned recipient record assembly
```

On `main`, complete recipient records can retain role, display name, address, and address kind when all validated diagnostics are supplied. The production reporting path does not yet generate and publish those complete records in one public-fixture run.

## Extraction and output boundaries

The parser layer owns byte interpretation and validated evidence. The extraction layer converts validated evidence into domain records. The output layer serialises records and raw body/attachment artefacts without reinterpreting PST bytes.

```text
Parser evidence
  -> extraction/domain records
  -> output contract
  -> TAR + JSONL
```

Diagnostics are not automatically part of the canonical data contract. A diagnostic becomes output data only after its semantic meaning, completeness behaviour, and stable record shape are explicitly validated.

## Canonical output

```text
<output-root>/
  run_summary.json
  progress.jsonl
  archives/
    <pst-id>_000001.tar
```

A TAR shard can contain:

```text
_pstfast/
  summary.json
  manifest.jsonl
  errors.jsonl
  folder_inventory.jsonl
  extraction_warnings.jsonl
  run_config.json

data/
  folders.jsonl
  messages.jsonl
  recipients.jsonl
  message_references.jsonl
  bodies.jsonl
  attachments.jsonl
  selected_mapi_properties.jsonl
bodies/
attachments/
```

The presence of a file family in the contract does not imply complete extraction coverage for every PST layout.

## Safety model

- All offsets, lengths, counts, row widths, and reference arithmetic are bounded.
- Ambiguous candidates remain unavailable rather than being selected heuristically.
- Internal table properties are excluded from user-readable value selection.
- Property identity and type are preserved through semantic decoding.
- Row-aligned records require equal complete evidence sequences.
- Partial values are suppressed after any validation failure.
- Cycle and depth guards bound recursive traversal.
- Public diagnostics exclude private payload bytes.

## Current limitations

- Fixture evidence is narrow and does not establish general PST compatibility.
- ANSI PST support and uncommon layouts remain incomplete.
- Attachment output is zero on the public fixture.
- Complete recipient records are not yet emitted through the production run.
- Structured output is not yet sufficient to claim lossless or RFC-complete EML reconstruction.

## Evolution path

The active architecture should evolve by wiring already validated evidence into complete extraction behaviours. New abstractions are justified only when they unlock an observable field or remove a demonstrated correctness blocker.
