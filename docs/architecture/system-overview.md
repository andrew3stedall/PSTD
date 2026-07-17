# PSTD System Overview

_Last reviewed: 17 July 2026._

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
| Heap-on-Node and BTH | Heap allocation lookup, B-tree-on-heap maps, and exact PtypObject HNID preservation. | Implemented and reused by property, table, and embedded-object paths |
| Property Context | Selected MAPI property lookup and decoding. | Implemented for the validated subset |
| Table Context | TCINFO, descriptors, row-index BTH, row payload resolution, row addressing, bitmap evidence, and value projection. | Implemented through bounded fixed-width and recipient string paths |
| Semantic extraction | Folder/message classification, bodies, recipient roles/names/addresses, threading, attachment evidence. | Partial but materially functional |
| Reporting | Bounded diagnostics, progress artifacts, structured output records. | Implemented for the validated recipient, body, and attachment paths |

## Current Table Context recipient flow

```text
validated TC heap
  -> TCINFO descriptors
  -> row-index BTH
  -> subnode-backed or heap-backed row payload
  -> validated row widths, offsets, and bitmaps
  -> property descriptor selection
  -> fixed-width recipient role values
  -> variable-width recipient name/address HNIDs
  -> Heap-on-Node string resolution
  -> authoritative property diagnostics
  -> row-aligned recipient record assembly
```

The original fixture publishes four complete recipient records from subnode-backed rows. Vertical 32 applies the same validated pipeline to heap-backed row allocations and publishes eight directly attributed Tika top-level recipients. Production ownership is restricted to direct recipient-table NIDs in each message root; Vertical 34 invokes that same selection on an isolated child subtree and publishes its ninth recipient under the child key.

## Embedded-message flow

```text
method-5 attachment Property Context
  -> preserved PtypObject HID
  -> exact 8-byte Nid + ulSize allocation
  -> normal-message NID validation
  -> exactly one NID match in the outer message's loaded subnode scope
  -> child Property Context + isolated child subnode subtree
  -> separate message/body/direct-recipient records
  -> parent attachment embedded_message_key link
```

Zero, duplicate, malformed, wrong-type, missing-block, or invalid-heap candidates produce no child. Nested child attachments and generated EML payload materialisation remain separate boundaries.

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
- Only one method-`5` embedded layout is validated; nested child attachments and recursive materialisation remain incomplete.
- The Tika child is structured but not yet emitted through the attachmentless plain-text EML path.
- Structured output is not yet sufficient to claim lossless or RFC-complete EML reconstruction across arbitrary PSTs.

## Evolution path

The active architecture should evolve by wiring already validated evidence into complete extraction behaviours. New abstractions are justified only when they unlock an observable field or remove a demonstrated correctness blocker.
