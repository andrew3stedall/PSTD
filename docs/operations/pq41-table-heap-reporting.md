# PQ41 — Aggregate table-heap reporting

## Decision

PQ40 proved that a single Heap-on-Node table context can resolve TCINFO, traverse the row-index BTH, and validate heap-backed row references. The next requirement is not column decoding. The missing evidence is whether real recursive subnode payloads contain table heaps that reach this path, and where resolution fails when they do not.

PQ41 therefore adds a bounded aggregation layer over decoded payload blocks. It identifies Heap-on-Node table-context payloads by their validated heap signature and client signature, invokes the existing PQ40 resolver, and preserves per-block success or exact failure evidence.

## Implemented reporting

The aggregate report records:

- decoded payload count;
- detected table-heap count;
- successful and failed TC heap resolutions;
- total discovered columns;
- total row references;
- in-bounds and out-of-bounds row-reference totals;
- table heaps whose rows require NID/subnode resolution;
- per-block BID, payload length, status, and exact error.

The implementation is intentionally side-effect free. It does not change extraction output and does not convert unvalidated row bytes into message properties.

## Evidence required from CI and the next public PST run

- Rust build, tests, Clippy, and rustfmt remain green.
- Existing Python, Docker, CLI, and public PST fixture jobs remain green.
- A following integration PQ should call `report_table_heaps` from the recursive subnode progress path and place the aggregate fields in the fixture artifact.
- The artifact must distinguish no reachable table heap from a reachable but unsupported heap. A zero count alone is not sufficient evidence.

## Extraction impact

No additional EML output is expected. PQ41 creates the structured reporting boundary required to expose real-fixture TCINFO and row-index evidence without expanding the existing status-string parser further.

## Unresolved risks

- The new aggregate function is exported but is not yet invoked by the CLI/public-fixture reporting path.
- NID-backed row matrices still require subnode-tree lookup.
- In-bounds offsets still require row-width and column-bound validation before decoding.
- Multi-page HID page-index handling remains unsupported.
- Real BTH key/value widths may differ from the currently supported four-byte row-index shape.

## Proposed PQ42

Wire `report_table_heaps` into the recursive subnode inspection/progress report and fixture artifact. Record table-heap counts, exact per-block failures, row-storage kind, and validated row-reference totals from the public PST. Only begin row materialisation if the fixture produces at least one resolved table heap with in-bounds references.
