# Active implementation checkpoint

- Issue: #622 (ROBUST-01); no parent epic.
- Branch: t3/issue-622; PR not yet created; baseline 0e58558.
- Scope: resolve Property Context variable-size HNIDs using owning subnodes, preserve empty values, fail closed on missing data. Excludes table-row #623 and unrelated robustness gaps.
- Established: bth::property_context_entry currently falls back to literal HNID bytes; synthetic LargeBody, ZeroLengthAttachment and missing attachment blocks expose this.
- Source boundary: src/pst/bth.rs, property_context.rs and production consumers; tests/synthetic_pst.rs.
- Tests: `cargo test --test synthetic_pst`: 4 passed, including exact LargeBody/ZeroLengthAttachment and missing-block no-payload assertions. Resolver unit tests compiling; full gate pending.
- Blockers: none known. Open draft #611 overlaps generic property resolution; avoid incorporating its broader scope.

## Next exact change
Run resolver unit tests and full gate; inspect any public-fixture delta. Source now preserves unresolved_hnid and status, resolves PC values in owning namespaces with bounded data-tree loading, and guards binary consumers. Embedded child uses the same node loader.
