# Active implementation checkpoint

- Issue: #622 (ROBUST-01); no parent epic.
- Branch: t3/issue-622; PR not yet created; latest implementation 10f60a9 (baseline 0e58558).
- Scope: resolve Property Context variable-size HNIDs using owning subnodes, preserve empty values, fail closed on missing data. Excludes table-row #623 and unrelated robustness gaps.
- Established: bth::property_context_entry currently falls back to literal HNID bytes; synthetic LargeBody, ZeroLengthAttachment and missing attachment blocks expose this.
- Source boundary: src/pst/bth.rs, property_context.rs and production consumers; tests/synthetic_pst.rs.
- Tests: `cargo test --test synthetic_pst`: 4 passed, including exact LargeBody/ZeroLengthAttachment and missing-block no-payload assertions. Resolver unit tests: 2 passed (ownership, missing/duplicate refs, invalid HID, indexed namespace, cycles and truncated stream). Full unit suite: 374 passed after restoring the shared Table Context reference path. Full integration gate pending. Strict local clippy blocked by pre-existing too_many_arguments and newer unknown-lint declarations on local Rust 1.94; CI uses stable and repository allowance.
- Pending increment: property_source.rs accepts unrelated empty sibling entries and compares stored keys without truncation; regression added. Focused rerun compiling in /tmp/pstd-622-target because prior workspace target hit OS error 38.
- Blockers: none known. Open draft #611 overlaps generic property resolution; avoid incorporating its broader scope.

## Next exact change
Run resolver regression, commit sibling-handling increment, then complete full gate and exact-head CI; preserve restored Table Context reference behavior (BIN PC payloads bypass second HNID interpretation). inspect any public-fixture delta. Source now preserves unresolved_hnid and status, resolves PC values in owning namespaces with bounded data-tree loading, and guards binary consumers. Embedded child uses the same node loader.
