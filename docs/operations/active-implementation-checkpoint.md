# Active Implementation Checkpoint

Updated: 2026-11-13

## Active delivery

- Issue: #614 — [PERF-02] Constant-time NBT/BBT lookups, positional reads and lean release build
- Parent epic: none
- Branch: `t3/optimize-extraction-performance`
- PR: not opened
- Base: `a9fdc27`
- Latest durable implementation commit: this commit
- Validation state: focused-green

## Scope

Output-preserving speed and footprint work: O(1) NBT/BBT lookups, positional reads,
per-reader crypt-method cache, release profile and trimmed dependency features.

### Out of scope

- B-tree page limit and the 1,000-message cap (next slice).
- Streaming/memory architecture, parser semantics, output contract.

## Established conclusions — do not re-derive

- `sort` and `tail` reading from a pipe hang in the local PRoot sandbox; redirect to files.
- Local Rust is 1.94; clippy needs `-A unknown_lints` locally (CI toolchain is newer).
- `zip` needs the `deflate` feature; `deflate-flate2` alone does not link `flate2`.
- Fixture outputs are byte-identical to `a9fdc27` except `duration_seconds`.

## Current source boundary

- `src/pst/lookup_index.rs` — `FirstKeyIndex`
- `src/pst/bbt.rs`, `src/pst/nbt.rs` — `lookup`, `lookup_index` field
- `src/pst/reader.rs` — `read_at`, `crypt_method_or_load`
- `src/pst/payload.rs` — `load_payload_block`
- `Cargo.toml` — release profile, dependency features

## Focused validation

- PASS: `cargo fmt --check`
- PASS: `cargo clippy --all-targets --all-features -- -D warnings -A clippy::too_many_arguments -A unknown_lints`
- PASS: `cargo test --all` — 19 suites ok, 359 lib tests
- PASS: fixture output equivalence against baseline binary
- NOT RUN: docker build, Python CLI check — left to CI

## Blockers / unresolved evidence

- None

## Next exact change

Open the PR, obtain fresh-context review, address findings, delete this checkpoint
and squash merge once the exact head is green.
