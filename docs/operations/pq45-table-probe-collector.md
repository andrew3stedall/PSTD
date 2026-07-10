# PQ45 — Collect table probes across extraction branches

## Decision

PQ44 added bounded run-level aggregation, but `metadata.rs` has two separate recursive subnode paths: the message-level probe for messages without the attachment flag and the attachment path. Directly building vectors and aggregating them independently at both sites would duplicate lifecycle logic and make it easier to omit one path from the final report.

PQ45 therefore adds a small collector boundary before modifying the large extraction function. Both paths can record an attributed `TcSubnodeProbeReport` into one collector, which is consumed exactly once to produce the existing bounded `TcRunAggregateReport`.

## Requirements implemented

- Add a default-constructible run collector for attributed table probes.
- Allow probes from independent extraction branches to be recorded incrementally.
- Expose the current probe count for instrumentation and tests.
- Consume the collector into the existing PQ44 aggregate without changing counter or diagnostic semantics.
- Verify that non-table and failed-table probes from separate paths are combined correctly.

## Before and after comparison

| Measure | Before PQ45 | After PQ45 |
|---|---:|---:|
| Run-level aggregate function | Available | Available |
| Incremental collection across extraction branches | Not available | Available |
| Single final aggregation point | Call-site responsibility | Enforced by collector consumption |
| Node/BID failure attribution | Available | Preserved |
| Diagnostic cap | 16 fragments | 16 fragments |
| Folders/messages/bodies/attachments/EML output | Existing baseline | Unchanged |
| Public PST table counters in extraction status | Not emitted | Still pending engine wiring |

PQ45 is a control-flow safety milestone. It does not claim additional extraction coverage.

## Tests

The new unit test records one decoded non-table probe and one failed table-heap probe through the collector, then verifies:

- both probes are counted;
- both decoded payload paths contribute;
- only the actual table-heap probe increments table counters;
- failed table evidence retains its originating message node ID;
- final aggregation remains compatible with the PQ44 status format.

The repository CI remains the acceptance gate for build, unit tests, Clippy, rustfmt, Python wrapper, Docker, CLI smoke tests, and the public PST fixture.

## Extraction position

The table-context path now provides:

1. recursive payload discovery;
2. table-heap detection;
3. TCINFO resolution;
4. row-index BTH traversal;
5. row-storage classification;
6. row-reference bounds validation;
7. per-probe node/BID attribution;
8. bounded run-level aggregation;
9. incremental collection across both extraction branches.

No row or column materialisation occurs yet.

## Risks

- `metadata.rs` still does not instantiate the collector or record probes.
- The public fixture has not proven that any real recursive payload contains a valid table-context heap.
- NID-backed row storage remains unresolved.
- In-bounds row references do not prove row width or column boundaries.
- Multi-page HID page indexes remain unsupported.
- The collector owns all probe reports until final aggregation; current parser limits bound this, but future larger runs may require streaming counters.

## Proposed PQ46

Instantiate `TcRunProbeCollector` once near the existing extraction counters. At both recursive subnode call sites, call `report_subnode_table_heaps(reference, &loaded_subnodes.payloads)` and record the result. After message processing, consume the collector, append its bounded progress status to the final extraction status, and add an issue record when table heaps are detected.

The public fixture comparison must report before and after values for:

- folders and messages discovered/extracted;
- body payloads, attachment payloads, attachment records, and output bytes;
- recursive probes and decoded payloads;
- probes containing table heaps;
- resolved and failed table heaps;
- total, in-bounds, and out-of-bounds row references;
- attributed message node IDs and subnode BIDs for failures.

Row materialisation remains blocked until a real fixture produces at least one resolved table heap with an in-bounds row reference.
