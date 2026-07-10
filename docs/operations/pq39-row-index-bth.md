# PQ39: Row-Index BTH Traversal

## Decision

PQ38 proved that a table-context Heap-on-Node payload can resolve `hidUserRoot`, parse `TCINFO`, and reach the referenced `row_index_hid`. The next bounded step is to parse the row-index BTH itself without yet decoding row payload columns.

## Implemented scope

- Resolve a supplied row-index BTH header HID from an already parsed Heap-on-Node.
- Validate the `0xb5` BTH header signature.
- Require the table row-index shape of four-byte row keys and four-byte row references.
- Enforce a maximum of eight index levels.
- Traverse leaf and intermediate allocations recursively.
- Detect allocation cycles.
- Cap emitted row-index entries at 4,096 and report truncation explicitly.
- Emit structured row-key and row-reference diagnostics.

## Evidence

Synthetic tests cover:

1. A leaf BTH containing two row keys and row references.
2. A one-level indexed BTH resolving a child leaf allocation.
3. Rejection of unsupported key/value widths.

The tests prove that the parser can move from a resolved `row_index_hid` to bounded row-key/reference output while preserving malformed-layout failures as explicit parser errors.

## Extraction impact

PQ39 still does not emit message fields or EML content. It advances extraction from table schema discovery to row-location discovery. The row references are now available for the next stage to interpret against heap-backed or subnode-backed row storage.

This implementation is intentionally separate from property-context BTH decoding because table row-index values are row references, not property HNIDs. Reusing the property-context value transformation would risk corrupting row offsets.

## CI evidence required

The pull request must pass:

- Rust build and unit tests;
- Clippy with warnings denied;
- rustfmt check;
- Python wrapper smoke test;
- Docker build;
- CLI and public PST progress fixture.

The public fixture currently validates regression safety. It does not yet prove that real table-context payloads invoke the new row-index parser, because pipeline wiring remains a subsequent change.

## Remaining risks

- The row-index parser is not yet invoked by the end-to-end message extraction pipeline.
- Row references have not yet been validated against the `rows_hnid` storage allocation.
- NID-backed row storage still requires subnode lookup.
- Multi-page HID page-index bits remain unsupported by the current Heap-on-Node model.
- Real PST evidence may reveal row-index key/value shapes or allocation layouts not represented by the synthetic fixtures.

## Proposed PQ40

Wire TC heap resolution and row-index traversal into table-context classification and progress reporting. For each reachable table context, report:

- resolved TCINFO roots;
- parsed row-index BTH headers;
- row-key/reference counts;
- heap-backed versus subnode-backed row storage;
- row references that fall within or outside the resolved row-data allocation;
- explicit failure reasons from real public PST payloads.

PQ40 should prioritize end-to-end observability and reference validation before decoding individual row columns.
