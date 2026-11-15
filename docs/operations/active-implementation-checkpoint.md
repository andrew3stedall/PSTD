# Active implementation checkpoint

- Issue: #633 [ROBUST-07] legacy flat Property Context path keeps evidence-selected recipients (follow-up to #623/#629, PR #632).
- Branch: `t3/issue-633-legacy-pc-recipients`. PR: not yet opened.

## Established conclusions

- In `tika-testPST.pst` the two legacy-path messages (`node_200044`, data BID `0x43e`; `node_2000c4`, data BID `0x3e6`) have internal BIDs: their Property Context heap is an XBLOCK data tree. The single-block heap probe scans the XBLOCK bytes, finds no heap, and falls back to the flat BTH parser, so the messages lose subject, sender, body and table-read recipients.
- Fix: data-tree Property Contexts are read by structure (`src/pst/pc_tree.rs`, reusing `tc_reader::{load_data_blocks, heap_allocation}`); an internal BID is never probed heuristically. A failing tree yields no properties and status `data_tree_property_context_unreadable`.
- With every real message PC read as a heap, the evidence-selected recipient fallback in `engine/metadata.rs::message_recipient_records` is removed: no readable recipient table means no recipients.

## Scope

In: `pc_tree.rs`, `node_payload.rs` dispatch, recipient fallback removal, Tika workflow expectations, docs.
Out: ANSI/4K trailer checks, embedded-message attachment reconciliation, deleting the now-unused evidence recipient modules.

## Tests

- Local: pending (`cargo test -j 2 --lib pc_tree`, `cargo test -j 2 --test synthetic_pst`).
- CI: pending.

## Next exact change

Run focused tests; open draft PR; read Tika workflow failures for the new expected output (two messages gain metadata; `Hong-Thai Nguyen` now from the table) and update expectations.
