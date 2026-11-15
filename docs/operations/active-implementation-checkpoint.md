# Active implementation checkpoint

- Issue: #633 [ROBUST-07] legacy flat Property Context path keeps evidence-selected recipients (follow-up to #623/#629, PR #632).
- Branch: `t3/issue-633-legacy-pc-recipients`. PR: #637 (draft).

## Established conclusions

- In `tika-testPST.pst` the two legacy-path messages (`node_200044`, data BID `0x43e`; `node_2000c4`, data BID `0x3e6`) have internal BIDs: their Property Context heap is an XBLOCK data tree. The single-block heap probe scans the XBLOCK bytes, finds no heap, and falls back to the flat BTH parser, so the messages lose subject, sender, body and table-read recipients.
- Fix: data-tree Property Contexts are read by structure (`src/pst/pc_tree.rs`, reusing `tc_reader::{load_data_blocks, heap_allocation}`); an internal BID is never probed heuristically. A failing tree yields no properties and status `data_tree_property_context_unreadable`.
- With every real message PC read as a heap, the evidence-selected recipient fallback in `engine/metadata.rs::message_recipient_records` is removed: no readable recipient table means no recipients.

## Scope

In: `pc_tree.rs`, `node_payload.rs` dispatch, recipient fallback removal, Tika workflow expectations, docs.
Out: ANSI/4K trailer checks, embedded-message attachment reconciliation, deleting the now-unused evidence recipient modules.

- The data-tree path requires both an internal BID and an XBLOCK/XXBLOCK header: the deterministic ANSI fixtures (`tools/ansi_fixture.rs`) label flat data blocks with internal BIDs (0x102, 0x10a), a spec deviation left for a follow-up.
- User decision: the ANSI stage-B flat synthetic message now expects zero recipient records (no readable recipient table); its EML `To` still comes from the message's own transport headers. Follow-up: make the ANSI fixtures spec-valid.
- Recovered: Tika `msg_d7325f9579306ba3` and `msg_4bc9d7bd77cbe661` gain subject/sender; Hong-Thai Nguyen is SMTP from the table; body-types messages `msg_41a4...`, `msg_5745...`, `msg_6e55...` gain text (and one RTF) bodies. Tika data-tree messages' text/HTML bodies stay present-but-unresolved, like existing `msg_33144...`/`msg_5e70...` (pre-existing large-body gap).

## Tests

- Local builds exceed the memory budget; rely on PR CI.
- Tika attachment and body-types workflow checks re-run locally against the fd92d6f CI artifacts: both pass with the updated expectations.

## Next exact change

Wait for CI on the stage-b expectation commit; if green, update README/changelog/project-status (ROBUST-07), delete this checkpoint, mark ready and squash-merge.
