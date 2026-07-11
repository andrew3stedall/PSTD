# PQ47 — Table-probe finalization boundary

## Context

PQ46 merged after CI run 408 passed. It provides one shared operation for recording recursively decoded subnode payloads into the attributed table-context collector.

Directly wiring that collector into `metadata.rs` still required three distinct concerns at once: finishing the collector, creating a bounded issue record only when table heaps exist, and appending the aggregate status to the run status. PQ47 isolates those finalization rules so the extraction-engine patch can remain mechanical and consistent.

## Revised requirement

PQ47 introduces a finalization boundary that:

- consumes `TcRunProbeCollector` exactly once;
- retains the existing bounded PQ44 aggregate and diagnostic cap;
- emits no issue for empty or non-table probes;
- emits one run-scoped issue when table-context evidence exists;
- preserves node ID, root subnode BID, resolution failures, row-reference totals, and truncation state in the issue and progress fragment.

This PQ deliberately does not alter extraction control flow or output counts. The next PQ must perform the remaining engine wiring rather than adding another reporting abstraction.

## Implementation

`src/pst/tc_extraction_reporting.rs` adds:

- `TcExtractionProbeSummary`;
- `finalize_table_probe_collection`;
- tests for non-table suppression and attributed failure evidence.

The module is exported from `src/pst/mod.rs`.

## Before-versus-after comparison

| Measure | Before PQ47 | After PQ47 |
|---|---:|---:|
| Shared recursive-load recording boundary | Available | Available |
| Single collector finalization operation | Call-site responsibility | Available |
| Issue suppression when no table heap exists | Call-site responsibility | Enforced |
| Bounded attributed issue when table evidence exists | Missing | Available |
| Diagnostic cap | 16 fragments | 16 fragments |
| Extracted folders/messages | Existing baseline | Unchanged |
| Body/attachment payloads | Existing baseline | Unchanged |
| EML files/output bytes | Existing baseline | Unchanged |
| Public-fixture table counters | Not emitted | Still pending engine wiring |

## Validation requirements

The branch must pass:

- Rust build and unit tests;
- Clippy with warnings denied;
- rustfmt;
- Python wrapper smoke test;
- Docker build;
- CLI smoke tests;
- public PST progress fixture.

## Remaining risks

- `metadata.rs` does not yet instantiate, populate, finish, or publish the collector.
- The public fixture has not yet established that a real recursive payload contains a resolvable table-context heap.
- NID-backed row matrices still need subnode-tree resolution.
- In-bounds offsets do not prove complete rows or valid column boundaries.
- Multi-page HID page indexes remain unsupported.

## Required PQ48 scope

PQ48 must be the engine integration step:

1. instantiate one `TcRunProbeCollector` near existing run counters;
2. call `record_subnode_payload_probe` after each successful recursive subnode load in both message paths;
3. call `finalize_table_probe_collection` after message processing;
4. append its optional issue to `issues`;
5. append `progress_status()` to the final extraction status;
6. publish public-fixture before/after evidence for folders, messages, bodies, attachments, EML files, output bytes, probes, decoded payloads, table heaps, row references, and attributed failures.

No further intermediate abstraction should be added before this wiring is completed.
