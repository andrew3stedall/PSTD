# Active implementation checkpoint

- Issue: #624 [ROBUST-03] block checksums are not verified; corrupt bytes are reported as extracted.
- Branch: `t3/issue-624-block-checksums` (from `main` at `aeb8129`).
- Scope: verify the Unicode block trailer (data size and CRC) in `load_payload_block`; withhold a failing block. Out of scope: ANSI and 4K-page trailers, trailer block id/signature, raw `load_block` callers (`src/pst/logical.rs`).
- Decision: a block that fails verification is withheld (error), not emitted with a status. The error text carries `block_checksum_mismatch` / `block_trailer_size_mismatch`, which reaches attachment records through the existing `data_tree_error=` status.
- Files: `src/pst/payload.rs` (`verify_block_trailer`, `block_crc32`, unit test), `src/pst/reader.rs` (cached Unicode-layout flag), `tests/synthetic_pst.rs` (checksum-mismatch and truncated-tree attachments asserted; gap constant removed).
- Tests: local `cargo fmt` and `cargo check --all-targets` pass. No tests run locally (memory limits); PR CI runs them.
- Risk to read in CI: hand-built Unicode test files or public fixtures whose blocks lack a valid trailer would now fail to load. If a public fixture fails, confirm against MS-PST before relaxing anything; if only hand-built test data fails, give that data valid trailers.
- Next exact change: read PR CI and act on the above; then delete this file and finalize the PR body.
