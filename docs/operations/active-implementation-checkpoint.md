# Active implementation checkpoint

Updated: 2026-09-08

## Active delivery

- Issue #600; parent epic #599; branch agent/coverage-foundation; draft PR #611.
- Latest durable implementation: 92146e5717497633a33b527b58a3523a8ab0f5ac.
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
- This typed PropertyContext API is not wired to production callers yet. No generic NID/data-tree implementation or coverage-completion claim.

## Source boundary

- src/pst/bth.rs: typed source records, inline/HID classification and focused regressions.
- src/pst/property_context.rs: typed decoding boundary and unresolved-reference regression.
- src/pst/node_payload.rs: next production caller. load_heap_bth_from_candidates currently flattens the BTH before decoding.
- src/pst/attachment_property_context.rs: later attachment migration; existing object/subnode special cases must not regress.
- src/pst/subnodes.rs and existing data-tree loader: generic NID source context to reuse later.

## Validation

- Actions run 34222579283, job 102048950884: "Format and test bounded implementation" succeeded on 1f049ea (BTH increment). Exact test count awaits logs.
- PropertyContext increment 92146e5 not yet validated.
- Local Rust/cargo absent; direct GitHub git network timed out. Use connector and existing .github/workflows/coverage-build-tools.yml fallback.
- Fallback now runs cargo fmt plus focused BTH and PropertyContext tests and commits only those Rust files. A stale runner push must fail normally; never force-push over new work.
- Full merge gate and fixture validation NOT RUN.

## Next exact change

Inspect focused job for the current branch head. Fix only reported compile/test failures and let the fallback persist formatting. Then wire node_payload.rs to keep typed entries from load_heap_bth_from_candidates and use PropertyContext::from_property_entries for heap-backed PC paths while preserving legacy flat parsing.

Before returning to broader scope, commit that integration and update this checkpoint. Then implement generic owner-scoped subnode/data-tree resolution using the existing bounded loaders. Do not search unrelated payloads globally or interpret all four-byte values as references.

## Merge-only work remaining

- Generic NID/data-tree loading, deterministic duplicate/cycle/missing/resource statuses.
- Production consumer migration and inline/HID/NID/data-tree regression fixtures.
- Current docs and diagnostics.
- Remove temporary coverage-build-tools.yml.
- Full exact-head validation, approved fixture deltas, review threads and final diff.
- Merge #611 only when #600 acceptance and all required gates actually pass.
