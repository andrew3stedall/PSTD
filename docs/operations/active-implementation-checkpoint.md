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

- `cargo fmt --check`: pass. `cargo test --all`: 20 suites pass, 0 failed (379 lib tests).
- `cargo clippy --all-targets --all-features -- -D warnings`: fails locally only on pre-existing code (`unknown lint: clippy::chunks_exact_to_as_chunks`, `too_many_arguments`) under the older local clippy; none in changed files. CI is the authority.
- Tika fixture local extract: 8 messages, 9 recipients (unchanged counts).

## Next exact change

1. Confirm CI on the PR head (clippy, Docker build, fixture workflows were not run locally).
2. Delete this checkpoint before merge.
