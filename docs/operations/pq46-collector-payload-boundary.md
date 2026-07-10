# PQ46 — collector payload recording boundary

## Decision

The previously proposed PQ46 scope was to edit both recursive subnode branches and the final status formatter in `src/engine/metadata.rs` in one change. Review of the current extraction code showed that the same conversion from a successful recursive load to an attributed table-context probe would otherwise be repeated at both call sites.

PQ46 therefore establishes one shared recording boundary first:

```rust
record_subnode_payload_probe(&mut collector, reference, &loaded_subnodes.payloads)
```

The next engine integration can use this identical operation in the attachment and non-attachment paths, reducing duplicated attribution logic and keeping the remaining control-flow patch small.

## Implementation

Added `src/pst/tc_probe_collection.rs` with `record_subnode_payload_probe`.

The function:

- accepts the run-scoped `TcRunProbeCollector`;
- accepts the originating `SubnodeReference`;
- accepts recursively decoded payload blocks;
- creates the existing attributed `TcSubnodeProbeReport`;
- records it through the bounded PQ45 collector.

Tests verify that:

- ordinary decoded payloads do not create false table-heap counts;
- failed table heaps retain message-node and root-subnode-BID attribution.

## Before and after

| Measure | Before PQ46 | After PQ46 |
|---|---|---|
| Conversion from recursive load to attributed probe | Call sites must compose reporting operations | One shared function |
| Attachment/non-attachment consistency | Dependent on duplicated call-site code | Same recording boundary available to both |
| Node and root BID attribution | Available | Preserved |
| Bounded run aggregation | Available | Preserved |
| Extracted folders and messages | Existing baseline | Unchanged |
| Body and attachment payloads | Existing baseline | Unchanged |
| EML files and output bytes | Existing baseline | Unchanged |
| Public-fixture table counters | Not emitted | Not emitted yet |

PQ46 is deliberately output-neutral. It improves the safety of the remaining engine integration but does not claim additional extraction coverage.

## Evidence required from CI

- Rust build and unit tests;
- Clippy with warnings denied;
- rustfmt;
- Python wrapper smoke test;
- Docker build;
- CLI smoke tests;
- public PST progress fixture.

## Remaining risks

- `metadata.rs` does not yet instantiate the collector or call the new boundary.
- The public fixture has not demonstrated a successfully resolved real table heap.
- NID-backed row storage still requires subnode-tree lookup.
- In-bounds row references do not establish complete row width or valid column boundaries.
- Multi-page HID page indexes remain unsupported.

## Proposed PQ47

Wire the shared boundary into both successful recursive subnode load paths in `metadata.rs`, finish the collector once after message processing, append its bounded progress status to the extraction result, and add one issue record when table heaps are detected.

PQ47 must publish a direct before/after comparison for:

- folders and messages discovered and extracted;
- body payloads, attachment payloads, attachment records, EML files, and output bytes;
- recursive probes and decoded payloads;
- detected, resolved, and failed table heaps;
- total, in-bounds, and out-of-bounds row references;
- message node IDs and root subnode BIDs for failures.
