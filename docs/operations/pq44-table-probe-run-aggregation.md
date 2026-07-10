# PQ44 — Aggregate attributed table-heap probe evidence

## Decision

PQ43 made each recursive subnode probe attributable to a message node and root subnode BID, but the extraction engine still has no bounded run-level structure for combining those probes. Wiring raw per-probe strings directly into `metadata.rs` would duplicate counters across the attachment and non-attachment branches and could create an unbounded final status.

PQ44 therefore adds the run-level aggregation boundary first. The following PQ can integrate one aggregate into both recursive call sites with a materially smaller control-flow change.

## Requirements implemented

- Aggregate all `TcSubnodeProbeReport` values into one run report.
- Count probes, decoded probes, decoded payloads, probes containing table heaps, resolved and failed heaps, columns, row references, bounds results, and subnode-backed row storage.
- Retain node/BID-attributed diagnostics only for probes that contain table heaps.
- Cap diagnostic fragments at 16 while preserving uncapped aggregate counters.
- Distinguish no probes, no detected table heaps, complete resolution, partial resolution, and complete failure.
- Emit a deterministic semicolon-delimited PQ44 progress fragment.

## Before and after comparison

| Measure | Before PQ44 | After PQ44 |
|---|---:|---:|
| Attributed per-probe reports | Available | Available |
| Run-level probe aggregation | Not available | Available |
| Aggregate row-reference/bounds totals | Not available | Available |
| Bounded node/BID diagnostics | Not available | Maximum 16 fragments |
| Extraction output or EML count | Unchanged | Unchanged |
| Public fixture evidence | PQ43 structures were regression-tested only | PQ44 structure is ready for engine integration; fixture values remain pending |

This is an observability and safety milestone. It does not claim improved extraction coverage.

## Tests

Unit coverage verifies:

1. empty runs and decoded non-table probes remain distinct;
2. failed table heaps retain root node and root subnode BID attribution;
3. diagnostic fragments are capped without reducing aggregate failure counts.

The repository CI remains the acceptance gate for build, tests, Clippy, rustfmt, Python wrapper, Docker, CLI smoke tests, and the public PST fixture.

## Extraction position

The table-context path now provides:

1. recursive payload discovery;
2. table-heap detection;
3. TCINFO resolution;
4. row-index BTH traversal;
5. row-storage classification;
6. row-reference bounds validation;
7. per-payload aggregation;
8. deterministic progress rendering;
9. message-node and root-BID attribution;
10. bounded run-level aggregation.

No table row or column is materialised yet.

## Risks

- `metadata.rs` does not yet collect or emit PQ44 reports.
- The public PST fixture has not proven that any real recursive payload reaches a valid table-context heap.
- A reference inside the row-data allocation does not prove the row width or column boundaries are valid.
- NID-backed row storage still requires subnode lookup.
- Multi-page HID page indexes remain unsupported.
- The 16-diagnostic cap is intended for status safety; detailed artifacts may later need a separate structured output.

## Proposed PQ45

Integrate `report_subnode_table_heaps` at both recursive message-subnode call sites, collect the resulting reports, call `aggregate_subnode_table_probes`, and append the PQ44 progress fragment to the final extraction status.

The public fixture comparison must record before and after values for:

- folders and messages discovered/extracted;
- body and attachment payloads;
- output bytes;
- recursive probes and decoded blocks;
- probes containing table heaps;
- resolved and failed table heaps;
- discovered, in-bounds, and out-of-bounds row references;
- attributed failure node IDs and root subnode BIDs.

Row materialisation remains blocked unless the public fixture produces at least one resolved table heap with at least one in-bounds row reference.
