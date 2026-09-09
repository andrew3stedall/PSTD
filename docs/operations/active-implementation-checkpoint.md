# Active implementation checkpoint

Updated: 2026-09-09

## Active delivery

- Issue #600; parent #599; branch agent/coverage-foundation; draft PR #611.
- Latest durable source/test commit: 3778f42de2e742d61c34b700edcea9f098683908 (tested formatting of 2fe2f2b).
- Scope: generic property storage/reference resolution. Do not start #601–#610 yet.
- Older local /workspace/scratch/10c9672c1630/pstd contains separate uncommitted work; do not overwrite or use as live PR.

## Established conclusions

- Typed BTH leaves retain property identity, raw HNID and storage provenance.
- PropertyContext preserves sources and excludes unresolved references from semantic decoding.
- node_payload.rs resolves owner-scoped Unicode NIDs and data trees through PropertyNodeResolver.
- attachment_property_context.rs uses the resolver only with a unique immediate owner; object-method handling remains separate.
- Four-byte resolved binary bodies are accepted; unresolved references are not emitted as opaque content.
- BTH rejects cycles, repeated children, duplicate tags, truncated leaves/indexes and resource overruns.
- ANSI subnode layouts and nonzero heap page indices remain unsupported by this generic path.
- PR_HTML, PR_RTF_COMPRESSED and arbitrary binary property NID/data-tree tests already exist in node_payload.rs (bdcb66d). Do not recreate them.

## Validation

- 2fe2f2b: focused Actions 34347271757 / job 102451756040 PASSED: cargo fmt, 371 library tests (0 failed), all-target/all-feature Clippy with existing CI exception. Formatting committed as 3778f42.
- This checkpoint commit triggers CI on the formatted source. Next inspect its CI and fixture runs.

- bdcb66d: Actions 34301699216 / job 102309778638 passed all 369 library tests.
- That job then failed Clippy: two manual_is_multiple_of errors in bth.rs, plus 13 too_many_arguments diagnostics.
- ffaef38 fixes both BTH lint errors.
- 8d8ef58 aligns temporary workflow with the existing ci.yml exception for too_many_arguments. This is the existing CI policy; strict AGENTS command without the exception remains a documented gate discrepancy.
- 93c7921 extends the body test through RTF emission and verifies missing owners produce no HTML/RTF output.
- 4931552 rejects overlapping indexed subnode leaf ranges and external BIDs used as index blocks, with regressions.
- 00977a4 adds full attachment extraction tests for direct NID, data-tree, sibling NID isolation and duplicate owner rejection. Await the latest head focused workflow.
- Temporary workflow runs cargo fmt, cargo test --lib, Clippy, and persists formatting of named Rust files. Bot commits may need a subsequent connector commit to trigger exact-head CI.
- No local Rust toolchain; direct git network is unavailable. Use connector, not another environment bootstrap.
- Full clean-head merge gate and approved fixture deltas not yet verified.

## Next exact change

1. The focused workflow is green (371 tests); inspect CI and fixture runs triggered by this checkpoint commit on the formatted source.
2. Inspect actual fixture failures after lint/formatting succeeds, especially attachment and embedded-message fixtures. Preserve established object handling.
3. Body output and missing-reference checks are committed in 93c7921; fix their failures if any instead of recreating them.
4. Review remaining #600 acceptance boundaries: generic HID page support, ANSI owner context, deterministic failure categories and exported diagnostics. Do not claim #600 complete while these remain unresolved.

## Merge-only work

- Finish acceptance and current docs/diagnostics; preserve provenance.
- Remove temporary coverage-build-tools.yml.
- Resolve strict Clippy gate discrepancy with a scoped justified change, not blanket warning suppression.
- Run cleaned exact-head full CI, approved fixture comparisons, review threads and final diff.
- Merge #611 only when acceptance and required gates pass.
