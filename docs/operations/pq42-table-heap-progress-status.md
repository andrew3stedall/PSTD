# PQ42 table-heap progress status

## Decision

PQ41 introduced a structured aggregate over reachable table-context heap payloads, but the latest public-PST artifact contains no PQ41 counters. The report exists only as an internal Rust value, so broad extraction-path wiring would currently require duplicating a long and fragile status format in the engine.

PQ42 therefore establishes a bounded, deterministic status boundary on `TcHeapAggregateReport` before changing extraction control flow. The formatter preserves aggregate counts and per-BID failure evidence while escaping semicolons inside nested resolver statuses and errors. This keeps the existing semicolon-delimited progress contract parseable.

## Implementation

`TcHeapAggregateReport::progress_status` now emits:

- decoded payload count;
- detected, resolved, and failed table-heap counts;
- total column and row-reference counts;
- in-bounds and out-of-bounds reference counts;
- subnode-backed row-storage count;
- compact per-BID diagnostics containing payload length, resolution state, counts, status, and exact error evidence.

The diagnostic list is pipe-delimited and nested semicolons are replaced with commas. Empty reports explicitly emit `pq42_diagnostics=none`.

## Evidence from the preceding run

GitHub Actions run 395 passed Rust build/tests, Clippy, rustfmt, Python wrapper, Docker, CLI smoke tests, and the public PST fixture. Its `public-pst-progress` artifact still ends at PQ36 summaries and the run status contains no table-heap aggregate fields. This confirms that PQ41 was regression-safe but not externally observable.

The public fixture currently extracts one message, two body payloads, no attachments, and 26,448 output bytes. It recursively decodes four subnode blocks, including one heap context and one unsupported layout. The next run must prove whether that heap context reaches the table-context resolver rather than assuming it does.

## Safety

This PQ does not change extraction output or attempt row materialisation. It only provides a stable representation for the existing bounded report. Exact per-block errors remain visible, and no unvalidated row bytes are decoded.

## PQ43 requirements

Wire `report_table_heaps(&loaded_subnodes.payloads).progress_status()` into both message-level subnode probe paths and aggregate the resulting fields into the run status and public fixture artifact.

The following run must report:

- table heaps detected across recursively decoded payloads;
- successful and failed TCINFO resolutions;
- exact BID and payload length for failures;
- HID-backed versus NID-backed row storage;
- total, in-bounds, and out-of-bounds row references;
- whether the public fixture reaches at least one validated row-storage allocation.

Do not begin column materialisation unless real-fixture evidence includes at least one resolved table heap and at least one in-bounds row reference. If no table heap is reached, the following PQ should expand payload selection rather than relax structural admission rules.
