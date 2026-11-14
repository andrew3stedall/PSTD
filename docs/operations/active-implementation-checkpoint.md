# Active implementation checkpoint

- Issue: #625 [ROBUST-04] attachments with an unreadable property context or a dangling table row leave no record.
- Branch: `t3/issue-625-damaged-attachment-records` (from `main` at `48ffee3`).
- Scope: read the message attachment table (subnode `0x671`) by tag; emit a payload-less record for each row no emitted record covers. Out of scope: attachments inside embedded messages, #624 (checksums, truncated trees).
- Conclusions: reconciliation only adds records when table rows outnumber emitted records; rows pair with records by filename; ambiguous pairing reports the missing count without row metadata. The table is read only for messages whose Property Context is a resolved heap (`tc_source`).
- Files: `src/pst/attachments.rs` (`attachment_table_rows`, `records_for_uncovered_table_rows`, 2 unit tests), `src/engine/metadata.rs` (top-level message attachment block), `tests/synthetic_pst.rs` (Garbage and dangling-row attachments asserted; `DAMAGED_ATTACHMENT_GAP` now covers #624 only).
- Tests: local `cargo fmt` and `cargo check --all-targets` pass. Unit, synthetic and fixture tests have NOT been run locally (memory limits); they run in PR CI.
- Blockers: none known; CI result pending.
- Next exact change: read PR CI. If `tests/synthetic_pst.rs` fails for `attachment_garbage_properties` or `attachment_dangling_row`, inspect the emitted record for that message; if a fixture workflow's hard-coded attachment expectations change, verify the new records are real table rows before updating them. Then delete this file and finalize the PR body.
