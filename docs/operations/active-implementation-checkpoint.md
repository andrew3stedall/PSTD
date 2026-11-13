# Active Implementation Checkpoint

Updated: 2026-11-13

## Active delivery

- Issue: #616 — [SCALE-01] Remove silent 1,000-item caps and the 1,024-page B-tree limit
- Parent epic: none
- Branch: `t3/lift-extraction-caps`
- PR: not opened
- Base: `d21293a`
- Latest durable implementation commit: this commit
- Validation state: focused-green

## Scope

Replace the silent `.take(1000)` message/folder caps with `InputLimits::max_candidate_items`,
raise `ParserLimits::max_btree_pages` to 262,144 and emit warning diagnostics when a
limit is reached.

### Out of scope

- Streaming extraction and memory architecture.

## Established conclusions — do not re-derive

- `sort` and `tail` reading from a pipe hang in the local PRoot sandbox; redirect to files.
- Local Rust is 1.94; clippy needs `-A unknown_lints` locally.
- A B-tree page holds at most 20 entries of 24 bytes, so synthetic trees over 1,024
  pages need four levels (`deep_bbt` in `tests/m7_parser_depth.rs`).
- `cap_diagnostics` keeps the first records, so limit warnings must be first.

## Current source boundary

- `src/pst/limits.rs` — `ParserLimits::default`
- `src/engine/metadata.rs` — `extract_metadata_with_limits`, `discover_folder_hierarchy`, `limit_warning`
- `tests/m7_parser_depth.rs` — deep-tree tests

## Focused validation

- PASS: `cargo fmt --check`
- PASS: `cargo clippy --all-targets --all-features -- -D warnings -A clippy::too_many_arguments -A unknown_lints`
- PASS: `cargo test --all` — 19 suites ok
- NOT RUN: docker build, Python CLI check, fixture workflows — left to CI

## Blockers / unresolved evidence

- No PST larger than 2.3 MB is available locally; scale behaviour is covered by
  synthetic trees only.

## Next exact change

Open the PR, obtain fresh-context review, address findings, delete this checkpoint
and squash merge once the exact head is green.
