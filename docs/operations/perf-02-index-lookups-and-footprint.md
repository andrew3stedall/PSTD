# PERF-02: index lookups, positional reads and release footprint

_Point-in-time evidence, 13 November 2026. Issue #614._

## Gap

- `BbtIndex::lookup` and `NbtIndex::lookup` scanned the whole entry vector per call, so
  resolving every block of a PST cost O(blocks²).
- `PstByteReader::read_at` duplicated the descriptor, seeked and read for every read.
- `load_payload_block` re-read the header crypt method (two reads) for every block.
- The crate had no release profile and enabled unused dependency default features.

## Change

- `src/pst/lookup_index.rs` adds `FirstKeyIndex`, a lazily built key → first position
  map held by both indexes (`#[serde(skip)]`, clone starts empty). Small indexes
  (≤16 entries) still scan. A cache whose entry count no longer matches, or whose
  cached position no longer holds the key, falls back to the original scan.
- `read_at` uses `FileExt::read_exact_at` on Unix and `seek_read` on Windows.
- The crypt method is cached per `PstByteReader`; a failed read is not cached.
- `[profile.release]`: `lto = true`, `codegen-units = 1`, `strip = true`.
  `chrono` uses `now` + `serde` only; `tar` drops default `xattr`.

## Evidence

aarch64 Linux (PRoot sandbox, Rust 1.94.0). Syscalls are unusually expensive in this
sandbox, so end-to-end ratios are indicative rather than portable.

| Lookup benchmark entries | Linear ms | Indexed ms | Speedup |
|---:|---:|---:|---:|
| 1,000 | 0.509 | 0.083 | 6.1× |
| 10,000 | 49.288 | 0.729 | 67.6× |
| 100,000 | 4165.413 | 9.445 | 441.0× |

| Fixture extract (median of 15 alternating runs) | Baseline ms | New ms |
|---|---:|---:|
| `tika-testPST.pst` | 73.80 | 21.89 |
| `java-libpst-dist-list.pst` | 65.05 | 15.24 |
| `inline-cid.pst` | 19.99 | 8.05 |

Release binary: 4,028,208 → 2,692,392 bytes (−33%).

Canonical tar contents for `sample.pst`, the three upstream fixtures and
`inline-cid.pst` are byte-identical between baseline `a9fdc27` and this change after
normalizing run identifiers and timestamps; only `duration_seconds` differs.

Reproduce the lookup table with
`cargo run --release --example index_lookup_benchmark`.

## Not addressed

- B-tree traversal stopped at `max_btree_pages = 1024` and message extraction was
  capped at the first 1,000 candidates. Both were lifted afterwards under issue #616.
- Extraction still buffers all records and attachment payloads in memory.
