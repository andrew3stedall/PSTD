# Active implementation checkpoint

- Active issue: #626 [ROBUST-05] Folder nesting is not rebuilt.
- Branch: `t3/issue-626-folder-nesting` (from `main`, independent of PR #632).
- Scope: folder paths and `parent_folder_key` from the NBT `nidParent`. Out of scope: hierarchy-table cross-checks, child counts, #624, #625.

## Established conclusions

- `NbtEntry.parent_node_id` is parsed from the 4-byte `nidParent` after the three ids (both layouts).
- `folder_tree::apply_folder_nesting` runs at the end of folder discovery in `engine/metadata.rs`; the self-parented root adds no segment; unknown parents and loops stay flat.
- `tests/synthetic_pst.rs`: `FOLDER_NESTING_GAP` removed; full paths are compared.

## Tests

- Local: `cargo build` and `cargo check --all-targets` pass. Tests, clippy and fixtures are run by PR CI only (local runs exhaust memory).

## Next exact change

1. Read PR CI results; fix code failures.
2. Workflow files with flat-path expectations (`tika-attachment-fixture.yml`, possibly `ansi-stage-b.yml`) need updating; the agent token cannot push workflow files.
3. Delete this checkpoint before merge.
