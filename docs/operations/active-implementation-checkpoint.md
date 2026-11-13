# Active implementation checkpoint

- Issue: #619 `[PERF-03]` CPU hot spots found by profiling. No parent epic. Related: #618.
- Branch: `t3/perf-03-cpu-hot-spots`. PR: not opened yet.
- Scope: make profiled CPU hot paths cheap without changing any output byte.
- Out of scope: #618 items (optional output profiles, streaming, table-context diagnostics).

## Established conclusions

- Profile (300 in-process extractions of `tests/fixtures/upstream/tika-testPST.pst`, SIGPROF
  sampler in the session scratchpad): ~28% `status_counter` substring scans, ~9% unbuffered
  tar `write`, ~6% `hex::encode`, SHA-256 on the software backend on aarch64 (sha2 0.10).
- File reads are not a bottleneck: 47-135 reads per fixture.
- `marker_value` must equal `status.split(&format!("{key}{sep}")).nth(1)` then
  `.split([',', ';']).next()`, including ending at a repeated marker.

## Changed so far

- `src/pst/status_marker.rs` (new): `marker_value`; used by `status_counter` in
  `src/engine/runner.rs` and `status_counter`/`status_value` in `src/pst/subnodes.rs`.
- `src/hexenc.rs` (new): table-based `encode`; `hex` dependency removed.
- `src/output/tar_writer.rs`: 256 KiB `BufWriter`, explicit flush on finish.
- `Cargo.toml`: `sha2` 0.11 (runtime-detected aarch64 SHA-2), no default features.

## Tests

- `cargo test --lib -- status_marker hexenc`: 3 passed.
- `cargo build --all-targets`: ok. Full gate not yet run.

## Next exact change

Re-profile, fix further hot spots that keep output identical, then run fixture output
equivalence against the scratchpad baseline binary, the full gate, changelog, PR, Sol review.
