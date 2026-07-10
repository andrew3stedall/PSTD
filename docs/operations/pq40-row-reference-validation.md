# PQ40: Combined TC Heap and Row-Reference Validation

## Revised decision

PQ39 added a bounded row-index BTH parser, but it remained isolated from the TC heap resolver. The next safe step is to combine those primitives into one deterministic report before changing broad extraction control flow.

This is narrower than directly materialising table rows. The public fixture has proved one real Heap-on-Node table-context payload is reachable, but has not yet proved that its row storage is heap-backed, that the row-index shape is supported, or that row references are valid offsets. Decoding columns before those facts are known would repeat the false-layout problem corrected in PQ34-PQ36.

## Implemented scope

`resolve_tcinfo_from_heap` now:

- parses the Heap-on-Node and resolves `hidUserRoot`;
- parses the TCINFO schema and property tags;
- invokes the PQ39 row-index BTH traversal for `hidRowIndex`;
- preserves the row-index parse error instead of collapsing it into a generic unresolved state;
- resolves heap-backed `hnidRows` allocations without treating NIDs as HIDs;
- records row-data allocation length;
- counts discovered row references;
- counts references inside and outside the heap-backed row-data allocation;
- retains explicit status for NID-backed rows requiring subnode resolution.

## Test evidence

Synthetic coverage now verifies:

1. Two row references inside an eight-byte heap row-data allocation.
2. A mixed case with one valid reference and one out-of-bounds reference.
3. NID-backed row storage, where bounds validation is deliberately deferred and the subnode requirement is preserved.

The fixture uses real one-based HID allocation indexing. The TCINFO root, BTH header, BTH leaf, row data, and index allocation each occupy distinct heap allocations.

## Extraction impact

PQ40 does not emit new messages or EML files. It advances the table path from independent parser primitives to a single bounded validation result that can be safely surfaced by extraction reporting.

The report distinguishes these blockers:

- row-index BTH unresolved;
- TC index allocation unresolved;
- rows stored in a subnode NID;
- heap row-data HID unresolved;
- one or more row references outside the resolved row-data allocation;
- all discovered heap row references validated.

## CI evidence required

The pull request must pass:

- Rust build and unit tests;
- Clippy with warnings denied;
- rustfmt check;
- Python wrapper smoke test;
- Docker build;
- CLI smoke tests;
- public PST progress fixture.

The public fixture remains regression evidence for this PQ. The combined resolver is not yet invoked by the broad subnode classification path, so it cannot yet provide real-fixture row counts.

## Remaining risks

- The table-context classifier still reports only heap type and allocation count; it does not yet publish the combined PQ40 report.
- NID-backed row storage requires lookup in the node's subnode tree.
- A row reference being within the allocation is necessary but not sufficient: row width and per-column bounds still require validation.
- Multi-page HID page-index bits remain unsupported by the current Heap-on-Node model.
- The BTH parser currently requires four-byte keys and four-byte values and may reject valid real-world variants that need separate evidence.

## Proposed PQ41

Wire the combined `TcHeapResolutionReport` into table-context classification and extraction status reporting, without decoding columns. Public-fixture evidence should include:

- reachable TC heap count;
- TCINFO success and failure counts;
- row-index success and exact failure reasons;
- discovered row-reference totals;
- heap-backed versus NID-backed row storage;
- in-bounds and out-of-bounds reference totals;
- the block ID and payload length for each failed table heap.

Only after a real fixture produces validated row references should the following PQ calculate row widths and decode selected columns.
