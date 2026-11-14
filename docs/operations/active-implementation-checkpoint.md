# Active implementation checkpoint

- Active issue: #623 [ROBUST-02] Table rows stored in a subnode are not read. Also closes the recipient half of #629 (rows read by column tag).
- Branch: `t3/issue-623-table-row-reader`. PR: not opened yet.
- Scope: general Table Context reader; folder membership and recipients consume it.
- Out of scope: folder nesting (#626), checksums (#624), damaged attachments (#625), removing the legacy `tc_*` evidence pipeline (still used for diagnostics and as fallback when a message has no readable recipient table).

## Established conclusions

- `src/pst/tc_reader.rs` reads a TC by structure: multi-block heap (HID block index), multi-level row-index BTH, `hnidRows` as HID or subnode NID with whole rows per data block, cells by column tag gated by the existence bitmap. Fails closed.
- `property_source::find_subnode_entry` returns `(data bid, subnode bid)` for a NID in a subnode tree.
- `message_table_membership.rs` uses `read_row_ids` (row ids only, no row matrix needed).
- `recipients::message_recipients` reads subnode `0x692`; `engine/metadata.rs::message_recipient_records` prefers it and falls back to the evidence projection only when it returns `None`.

## Tests

- See latest commit message for the exact results.

## Next exact change

1. Set `RECIPIENT_GAP` and `LARGE_FOLDER_GAP` to `false` in `tests/synthetic_pst.rs`, run `cargo test --test synthetic_pst`.
2. Refresh exact fixture metrics that change because `tika-testPST.pst` now yields recipients.
3. Update README / project-status / progress log / changelog, run the full gate, open the PR.
