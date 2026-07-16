# PSTD Codebase Map

_Last reviewed: 16 July 2026._

## Purpose

Map the current repository to its runtime, parser, extraction, output, test, and documentation responsibilities.

## Top level

| Path | Responsibility |
|---|---|
| `Cargo.toml` | Rust crate, binary, dependencies, and feature configuration. |
| `src/main.rs` | Binary entrypoint. |
| `src/lib.rs` | Library exports. |
| `python/` | Thin Python operator wrapper. |
| `docker/` | Local container packaging. |
| `tests/` | Integration, regression, CLI, output-contract, and fixture-oriented tests. |
| `scripts/` | Public fixture progress and bounded diagnostic helpers. |
| `docs/` | Current guidance plus historical milestone/PQ/vertical evidence. |
| `.agents/skills/` | Repository-scoped planning and execution instructions. |
| `.github/workflows/` | CI, fixture validation, and artifact generation. |

## Command, configuration, and engine

| Path | Responsibility |
|---|---|
| `src/cli.rs` | `inspect`, `extract`, `batch`, and `version` command surface. |
| `src/config.rs` | Runtime configuration derived from CLI arguments. |
| `src/engine/` | Extraction orchestration and conversion of validated parser evidence into records. |
| `src/progress.rs` | Structured progress events. |
| `src/error.rs` | Shared error and status types. |

## Core PST storage and traversal

| Path | Responsibility |
|---|---|
| `src/pst/reader.rs` | Bounded random-access reads. |
| `src/pst/header.rs` | PST header and root metadata. |
| `src/pst/primitives.rs` | Typed PST identifiers and references. |
| `src/pst/binary.rs` | Checked little-endian decoding helpers. |
| `src/pst/trailer.rs` | Page and block trailers. |
| `src/pst/bbt.rs` | Block B-tree decoding and traversal. |
| `src/pst/nbt.rs` | Node B-tree decoding and traversal. |
| `src/pst/block.rs` | Bounded block loading and decoding boundaries. |
| `src/pst/logical.rs` | Logical node access. |
| `src/pst/node_payload.rs` | Node payload resolution. |
| `src/pst/subnodes.rs` | Subnode traversal and guards. |
| `src/pst/payload.rs` | Payload decoding and structural admission. |
| `src/pst/limits.rs` | Parser limits, depth limits, and safety bounds. |
| `src/pst/inspect.rs` | Inspect summary and diagnostics. |

## Heap, property, and table parsing

| Path | Responsibility |
|---|---|
| `src/pst/heap.rs` | Heap-on-Node allocation parsing and lookup. |
| `src/pst/bth.rs` | Generic B-tree-on-heap parsing. |
| `src/pst/property_context.rs` | Property Context parsing and selected property extraction. |
| `src/pst/table_context.rs` | General Table Context boundaries. |
| `src/pst/tcinfo.rs` | TCINFO root and column descriptors. |
| `src/pst/tc_bth.rs` | Table Context row-index BTH handling. |
| `src/pst/tc_heap.rs` | Validated Table Context heap resolution. |
| `src/pst/tc_subnode_rows.rs` | Shared validated row-storage resolution for subnode-backed and heap-backed matrices. |
| `src/pst/tc_row_payload_candidates.rs` | Bounded row-payload candidate selection. |
| `src/pst/tc_row_offsets.rs` | Absolute row offset derivation. |
| `src/pst/tc_row_mode.rs` | Validated direct/ordinal row-address mode selection. |
| `src/pst/tc_row_transport.rs` | Coupled row payload and validated offsets. |
| `src/pst/tc_row_resolution_transport.rs` | Bridge from row resolution to validated transport. |
| `src/pst/tc_row_transport_metadata.rs` | Bounded transport reporting. |
| `src/pst/tc_descriptor_evidence.rs` | Descriptor/bitmap evidence and mapping validation. |
| `src/pst/tc_fixed_width_evidence.rs` | Bounded fixed-width row evidence. |
| `src/pst/tc_fixed_width_projection.rs` | End-to-end supported scalar projection. |
| `src/pst/tc_fixed_width_diagnostic.rs` | Stable bounded diagnostic representation. |
| `src/pst/tc_property_classification.rs` | Authoritative classification of known Table Context properties. |

## Recipient extraction

| Path | Responsibility |
|---|---|
| `src/pst/recipients.rs` | Recipient domain helpers and output foundations. |
| `src/pst/tc_recipient_identity_reference.rs` | Extract recipient identity HNID references from validated rows. |
| `src/pst/tc_recipient_identity_string.rs` | Resolve and decode heap-resident recipient strings. |
| `src/pst/tc_recipient_identity_projection.rs` | Compose row transport, reference extraction, and string resolution. |
| `src/pst/tc_recipient_identity_diagnostic.rs` | Bounded recipient identity diagnostics and value-kind classification. |
| `src/pst/tc_recipient_records.rs` | Assemble row-aligned role/name records and complete role/name/address/address-kind records. |
| `src/pst/tc_complete_recipient_projection.rs` | Project display names and preferred addresses from one validated row source and assemble complete records. |
| `src/pst/tc_message_recipient_selection.rs` | Select exactly one directly attributed validated recipient projection per message. |
| `src/pst/tc_message_recipient_output.rs`, `tc_message_recipients.rs` | Convert selected projections into stable structured `RecipientRecord` rows. |
| `src/pst/tc_reporting.rs` | Production Table Context diagnostic integration. |
| `src/pst/tc_extraction_reporting.rs`, `tc_run_reporting.rs`, `tc_probe_collection.rs` | Aggregate and publish Table Context evidence. |

## Other extraction domains

| Path | Responsibility |
|---|---|
| `src/pst/folder_tree.rs`, `folders.rs` | Folder classification and records. |
| `src/pst/message_table.rs`, `message_metadata.rs`, `messages.rs` | Message discovery, metadata, and records. |
| `src/pst/mapi.rs` | Selected MAPI property registry and decoding. |
| `src/pst/attachments.rs`, `attachment_table.rs` | Attachment metadata/table foundations and payload evidence. |
| `src/pst/threading.rs` | Threading and reference helpers. |
| `src/pst/compatibility_m21.rs` | Fixture-backed compatibility classification retained under the `compatibility` module. |

## Output

| Path | Responsibility |
|---|---|
| `src/output/tar_writer.rs` | TAR shard creation. |
| `src/output/jsonl_writer.rs` | JSONL serialisation. |
| `src/output/metadata.rs` | Output record types. |
| `src/output/ids.rs` | Stable identifier generation. |
| `src/output/paths.rs` | Safe deterministic archive paths. |
| `src/output/summary.rs` | Extraction and run summaries. |

## Where to start

| Work | First files |
|---|---|
| Current capability or blocker | `README.md`, `docs/product/project-status.md` |
| Fixture evidence | `docs/operations/public-pst-progress-log.md`, workflow artifacts |
| Table Context parsing | `src/pst/tcinfo.rs`, `tc_heap.rs`, row transport modules |
| Recipient extraction | recipient identity modules, `tc_recipient_records.rs`, `tc_reporting.rs` |
| Structured output | `src/output/`, `docs/data/pstd-v1-output-contract-summary.md` |
| Validation | `docs/operations/local-validation.md`, `.github/workflows/` |

## Boundaries

Parser modules interpret bytes; extraction modules create semantic records; output modules serialise records. Python must not parse PST internals. Downstream systems must consume output rather than reopening PST files. Unsupported structures must remain explicit rather than being hidden by fallback heuristics.
