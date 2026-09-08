# Active implementation checkpoint

Updated: 2026-09-08

## Active delivery

- Issue #600; parent epic #599; branch agent/coverage-foundation; draft PR #611.
- Latest durable implementation: ab71506365ab4754a3afa373ca9aa5b127d5c3c6 (plus synthetic header correction).
- Previous implementation: c372cef34b962d22fe9009d78f215051535e9d18.
- Scope: generic property storage/reference resolution only. #601–#610 are not active scope.
- Existing older local checkout /workspace/scratch/10c9672c1630/pstd has unrelated uncommitted coverage work. Do not overwrite it or treat it as the live PR.

## Established conclusions

- BTH now exposes parse_property_context_with_sources returning (BthHeader, Vec<BthPropertyEntry>), without changing BthEntry/BthMap struct literals.
- PropertySource preserves prop_id, prop_type, raw value_hnid, optional HnidKind, and PropertyStorageStatus.
- Inline scalar types must not be resolved as heap references even when their bits match a HID.
- NIDs must not pass through permissive heap hid_index. Nonzero heap page indexes are unresolved at this single-page boundary.
- Existing flattened parser remains available. Typed records distinguish Inline, Heap, NodeUnresolved, HeapUnresolved, Null, ObjectReference.
- PropertyContext::from_property_entries accepts typed leaves and prevents semantic decoding of unresolved NID/HID/object references. It preserves raw reference bytes with HNID_UNRESOLVED status and counts them separately from decode errors.
- Typed PropertyContext API is wired to node_payload.rs. Owner-scoped Unicode PropertyNodeResolver is committed but not wired to this caller yet.

## Source boundary

- src/pst/bth.rs: typed source records, inline/HID classification and focused regressions.
- src/pst/property_context.rs: typed decoding boundary and unresolved-reference regression.
- src/pst/node_payload.rs: next production caller. load_heap_bth_from_candidates currently flattens the BTH before decoding.
- src/pst/attachment_property_context.rs: later attachment migration; existing object/subnode special cases must not regress.
- src/pst/subnodes.rs and existing data-tree loader: generic NID source context to reuse later.

## Validation

- Actions run 34222579283, job 102048950884: "Format and test bounded implementation" succeeded on 1f049ea (BTH increment). Exact test count awaits logs.
- Actions 34222832271 succeeded: 7 BTH, 11 PropertyContext, 4 node_payload tests. Formatting persisted as 3d505c6. New resolver tests await validation.
- Local Rust/cargo absent; direct GitHub git network timed out. Use connector and existing .github/workflows/coverage-build-tools.yml fallback.
- Fallback now runs cargo fmt plus focused BTH and PropertyContext tests and commits only those Rust files. A stale runner push must fail normally; never force-push over new work.
- Full merge gate and fixture validation NOT RUN.

## Next exact change

Validate current focused workflow. Node-backed resolution is now integrated in node_payload.rs with Subnode/DataTree statuses and owner/source BID provenance in PropertyContextParseReport.property_sources. Added end-to-end subnode subject extraction. Missing references preserve HNID_UNRESOLVED with resolver reason; total resolved bytes per PC are bounded.

Resolver tests: 4/5 passed in job 102204360537. Indexed-leaf test failed because its synthetic block at offset 512 overlapped crypt-method header offset 513. Fixed fixture start to 1024; rerun pending. Do not re-diagnose this as resolver decryption corruption.

Next after focused success: add HTML/RTF/data-tree production-path regressions, verify ANSI handling (current resolver is explicitly Unicode), and migrate safe attachment consumers while preserving object semantics. Then run required full/fixture gates. Remaining generic boundary hardening: BTH traversal currently truncates on limits; data-tree errors currently collapse to PayloadInvalid.

## Merge-only work remaining

- Generic NID/data-tree loading, deterministic duplicate/cycle/missing/resource statuses.
- Production consumer migration and inline/HID/NID/data-tree regression fixtures.
- Current docs and diagnostics.
- Remove temporary coverage-build-tools.yml.
- Full exact-head validation, approved fixture deltas, review threads and final diff.
- Merge #611 only when #600 acceptance and all required gates actually pass.
