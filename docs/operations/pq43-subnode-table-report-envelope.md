# PQ43 — Bind table-heap evidence to recursive subnode probes

## Decision

PQ42 established a deterministic aggregate table-heap status string, but it did not preserve which message-node subnode probe produced the payload set. Wiring the aggregate directly into both metadata branches would duplicate formatting and make cross-message evidence ambiguous.

PQ43 therefore introduces a single probe-level reporting envelope before the extraction-engine call sites are changed. The envelope binds the existing bounded table-heap report to:

- the root message node ID;
- the root subnode block ID;
- the number of recursively decoded payloads;
- an explicit empty, non-table, resolved, partially resolved, or unresolved state.

This is a narrower integration boundary than the earlier proposal, but it removes a structural ambiguity that would otherwise make public-fixture evidence difficult to attribute.

## Implementation

`report_subnode_table_heaps` accepts a `SubnodeReference` and the recursively decoded payloads for that probe. It returns `TcSubnodeProbeReport`, which embeds `TcHeapAggregateReport` and renders a semicolon-delimited progress fragment containing both PQ43 probe identity and the existing PQ42 counters.

The report distinguishes:

- `pq43_subnode_payloads_empty`;
- `pq43_no_table_heaps_detected`;
- `pq43_table_heaps_resolved`;
- `pq43_table_heaps_partially_resolved`;
- `pq43_table_heaps_unresolved`.

Unit coverage verifies that failed table heaps remain attributable to their root node and subnode BID, and that empty probes are not conflated with decoded payloads containing no table-context heap.

## Extraction impact

No row or EML output changes are expected in PQ43. This milestone creates the reusable evidence object needed for safe integration into both recursive subnode paths.

## Evidence required from CI

- Rust build, tests, Clippy, and rustfmt pass.
- Python and Docker smoke checks remain green.
- The public PST fixture remains regression-safe.
- New tests prove root node/BID attribution and empty-versus-non-table classification.

## Remaining risks

- The extraction engine does not yet invoke the new envelope.
- The public fixture has not yet demonstrated a real table-context heap reaching TCINFO resolution.
- NID-backed row data still requires subnode-tree lookup.
- In-bounds row references do not prove complete rows or valid column boundaries.
- Multi-page HID page-index handling remains unsupported.

## Proposed PQ44

Invoke `report_subnode_table_heaps(reference, &loaded_subnodes.payloads)` in both recursive message-subnode paths. Aggregate the resulting probe reports into the run status and issue records, while retaining a bounded number of per-probe diagnostics.

The public fixture must then establish:

- total recursive probes examined;
- probes with decoded payloads;
- probes containing table-context heaps;
- resolved and failed table heaps;
- row-reference and bounds totals;
- root node and subnode BID for each failure.

Row materialisation remains blocked until at least one real probe produces a resolved table heap and an in-bounds row reference.
