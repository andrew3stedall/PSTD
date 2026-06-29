# PSTD Codebase Map

## Purpose

Map the current source tree to the responsibilities introduced across M1-M3.

## Top-level files

| Path | Purpose |
|---|---|
| `Cargo.toml` | Rust crate metadata and dependencies. |
| `src/main.rs` | Minimal binary entrypoint. |
| `src/lib.rs` | Library module exports. |
| `README.md` | Project overview and start-here links. |
| `AGENTS.md` | Repository operating model and skill instructions. |

## Rust command and engine modules

| Path | Responsibility |
|---|---|
| `src/cli.rs` | Defines `pstd extract`, `pstd inspect`, and `pstd version`. |
| `src/config.rs` | Runtime config derived from CLI args. |
| `src/engine/runner.rs` | Orchestrates extraction and archive writing. |
| `src/engine/metadata.rs` | Metadata-only extraction path introduced in M3. |
| `src/progress.rs` | Structured progress event model. |
| `src/error.rs` | Shared error and status types. |

## PST modules

| Path | Responsibility | Current maturity |
|---|---|---|
| `src/pst/reader.rs` | Bounded random-access file reads. | Foundation |
| `src/pst/header.rs` | Basic PST header parsing and variant summary. | Foundation |
| `src/pst/primitives.rs` | Typed PST identifiers and references. | Foundation |
| `src/pst/binary.rs` | Little-endian parsing helpers. | Foundation |
| `src/pst/trailer.rs` | Page and block trailer parsing. | Foundation |
| `src/pst/bbt.rs` | Block B-tree page/index skeleton. | Skeleton |
| `src/pst/nbt.rs` | Node B-tree page/index skeleton. | Skeleton |
| `src/pst/block.rs` | Bounded block loading. | Foundation |
| `src/pst/inspect.rs` | `pstd inspect` summary generation. | Foundation |
| `src/pst/logical.rs` | Logical node loading boundary. | Foundation |
| `src/pst/heap.rs` | Heap-on-node parser foundation. | Foundation |
| `src/pst/bth.rs` | BTH map parser foundation. | Foundation |
| `src/pst/property_context.rs` | Property context scaffold. | Scaffold |
| `src/pst/table_context.rs` | Table context scaffold. | Scaffold |
| `src/pst/mapi.rs` | Selected MAPI property registry and value decoding. | Selected set only |
| `src/pst/folder_tree.rs` | Folder inventory/status output helpers. | Foundation |
| `src/pst/message_metadata.rs` | Message metadata/status row helpers. | Foundation |
| `src/pst/folders.rs` | Earlier placeholder/future module. | Placeholder |
| `src/pst/messages.rs` | Earlier placeholder/future module. | Placeholder |
| `src/pst/attachments.rs` | Earlier placeholder/future module. | Placeholder |

## Output modules

| Path | Responsibility |
|---|---|
| `src/output/tar_writer.rs` | Writes TAR shards. |
| `src/output/jsonl_writer.rs` | Writes newline-delimited JSON buffers. |
| `src/output/metadata.rs` | Output record structs for folders, messages, bodies, attachments, manifest, selected MAPI properties. |
| `src/output/ids.rs` | Stable ID helpers. |
| `src/output/paths.rs` | Safe archive path helpers. |
| `src/output/summary.rs` | Extraction summary record. |

## Python wrapper

| Path | Responsibility |
|---|---|
| `python/src/pstd/cli.py` | Finds/invokes the Rust `pstd` binary. |
| `python/src/pstd/__main__.py` | Supports `python -m pstd`. |
| `python/setup.py` | Minimal package scaffold. |

## Tests

| Path | Coverage |
|---|---|
| `tests/cli_smoke.rs` | Basic package/version smoke check. |
| `tests/output_contract.rs` | ID/path/JSONL/TAR writer smoke tests. |
| `tests/pst_binary.rs` | M2 byte reader/header/binary/trailer smoke tests. |
| `tests/m3.rs` and/or `tests/m3_metadata.rs` | M3 heap/BTH/table/MAPI smoke tests, depending on current branch state. |

## Important boundaries

- PST parsing must stay separate from output writing.
- CLI parsing must stay separate from extraction logic.
- Python must not parse PST internals.
- Snowflake/web/search must not be introduced into v1 parser milestones.
- Missing or unsupported PST structures should be recorded explicitly, not hidden.
