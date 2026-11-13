# Active implementation checkpoint

- Issue: #622 (ROBUST-01); no parent epic.
- Branch: t3/issue-622; PR not yet created; baseline 0e58558.
- Scope: resolve Property Context variable-size HNIDs using owning subnodes, preserve empty values, fail closed on missing data. Excludes table-row #623 and unrelated robustness gaps.
- Established: bth::property_context_entry currently falls back to literal HNID bytes; synthetic LargeBody, ZeroLengthAttachment and missing attachment blocks expose this.
- Source boundary: src/pst/bth.rs, property_context.rs and production consumers; tests/synthetic_pst.rs.
- Tests: none run yet.
- Blockers: none known. Open draft #611 overlaps generic property resolution; avoid incorporating its broader scope.

## Next exact change
Trace PC consumers and reuse existing bounded subnode/data-tree resolver for variable-size values; add focused synthetic regression assertions.
