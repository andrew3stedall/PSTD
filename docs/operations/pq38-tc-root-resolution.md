# PQ38: TC Heap Root Resolution

## Decision

PQ37 established bounded `TCINFO` parsing and HNID classification, but left the parsed structure disconnected from the enclosing Heap-on-Node allocation map. PQ38 closes that gap by resolving `hidUserRoot`, parsing the referenced `TCINFO` allocation, and validating the heap-backed HIDs referenced by the table context.

## Implemented scope

- Parse the enclosing Heap-on-Node payload.
- Resolve `HeapHeader.user_root` as a HID allocation.
- Parse the resolved allocation as `TCINFO`.
- Preserve all column property tags for later row decoding.
- Resolve `row_index_hid` and `index_hid` against the same heap.
- Resolve `rows_hnid` when it is a HID.
- Explicitly report when `rows_hnid` is an NID and therefore requires subnode resolution rather than heap lookup.
- Return a structured status instead of silently treating unresolved references as decoded table rows.

## Evidence

Synthetic heap fixtures cover both supported HNID branches:

1. Heap-backed rows: `hidUserRoot=0x40`, `row_index_hid=0x60`, `rows_hnid=0x80`, and `index_hid=0xa0` all resolve to bounded allocations.
2. Subnode-backed rows: `rows_hnid=0x74` is preserved as `HnidKind::NodeId`, with `rows_requires_subnode_resolution=true`.

The TCINFO fixture contains two real MAPI property tags (`0x001a0037` and `0x001f3001`) to verify that the resolved root exposes column metadata needed by the next stage.

## Extraction impact

PQ38 does not yet emit message rows or body text. It advances extraction by converting a table-context heap from a classified payload into a resolved schema root with validated row/index references. This removes ambiguity about whether the table root and referenced HIDs are reachable before implementing BTH row-index traversal.

## Remaining risks

- HID page-index bits are not yet interpreted across multi-page heaps; the current heap model resolves allocation indexes within the supplied payload.
- NID-backed `rows_hnid` values still require subnode lookup.
- `row_index_hid` and `index_hid` allocations are validated for reachability but not yet parsed as BTH structures.
- Public-fixture extraction must confirm that real table-context payloads reach this resolver; synthetic tests alone do not establish end-to-end coverage.

## Proposed PQ39

Implement bounded BTH traversal for the resolved `row_index_hid` allocation, emit row-key/row-offset diagnostics, and record whether `rows_hnid` is heap-backed or subnode-backed. The public PST progress artifact should report:

- resolved TC heap roots;
- parsed row-index BTH headers;
- discovered row keys;
- resolvable row payload references;
- failure reasons for unsupported BTH levels or external subnode rows.
