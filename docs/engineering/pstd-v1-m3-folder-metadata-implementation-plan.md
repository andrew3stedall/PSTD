# PSTD v1 M3 Folder and Metadata Implementation Plan

## Purpose

Define the implementation shape for the metadata extraction milestone.

## Target modules

```text
src/pst/
  logical.rs
  heap.rs
  bth.rs
  property_context.rs
  table_context.rs
  mapi.rs
  folder_tree.rs
  message_metadata.rs
```

Existing M2 modules should remain the byte/block foundation. M3 should add logical PST layers above them.

## Node and block access

M3 should expose a logical access boundary that can load node data through M2 NBT/BBT/block APIs.

Required behaviour:

- Load node data by `NodeId`.
- Return structured unsupported diagnostics when a node cannot be loaded.
- Keep multi-block and subnode edge cases explicit.

## Heap-on-node parser

Add a heap-on-node parser foundation that can decode heap metadata needed by property and table contexts.

## BTH parser

Add a BTH parser foundation for heap-backed maps used by property contexts and tables.

## Property context parser

Parse selected property contexts and produce typed property values where practical.

## Table context parser

Parse table contexts sufficiently to support folder hierarchy and message metadata rows.

## Selected MAPI properties

Start with a selected registry only. Decode enough fields for M3 metadata:

```text
subject
sender name
sender email when cheap
sent time
received time
created time
modified time
message class
importance
read status
message size
has attachments
folder display name
folder item counts
```

Recipients, full threading, bodies, and attachments remain later milestones.

## Folder hierarchy

Use table/property contexts to emit:

```text
folders.jsonl
_pstfast/folder_inventory.jsonl
```

## Message metadata

Emit `messages.jsonl` rows with available metadata and explicit status fields for missing/deferred content.

## CLI integration

Use a metadata-only extraction path such as:

```text
pstd extract --input archive.pst --output ./out --manifest-only
```

## Validation commands

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
pstd inspect --input <approved-small-fixture.pst>
pstd extract --input <approved-small-fixture.pst> --output <tmp-output> --manifest-only
```

Do not claim these passed unless they were actually run.
