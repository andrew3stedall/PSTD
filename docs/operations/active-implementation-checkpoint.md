# Active implementation checkpoint

- Issue: #622 follow-up; original fix merged as #630 (2b73d4b).
- Branch: t3/issue-622; PR: #631; latest implementation: 0e64136; main reconciliation: b81befc.
- Scope: repair property lookup with unrelated empty/wide sibling entries; validate full original fix. Excludes #623 and other robustness issues.
- Established: original #630 merged with failing Tika/content/embedded-graph fixture checks. Only source delta from main is property_source.rs and its regression. Table-row BIN references retain their existing interpretation.
- Source: src/pst/property_source.rs, owning_subnode_root/find_data_bid.
- Validation: cargo fmt --check passed. Resolver tests: 3 passed, 0 failed. Full suite passed (cargo test --all: exit 0); remaining gate running with CARGO_TARGET_DIR=/tmp/pstd-622-target. Docker unavailable locally; use CI. Prior baseline full suite passed; current fixture delta pending.
- Fixture conclusions: ANSI generator incorrectly heap-stored PT_LONG method/size values; fixed generator, exact metadata assertions retained. Tika DOCX recovered; update old diagnostic string expectation to resolved-PC status, retain exact bytes/hash checks.
- Blockers: no source blocker. Local Rust 1.94 has pre-existing unknown lint / too_many_arguments clippy differences; CI stable is authoritative.

## Next exact change
Inspect /tmp/622-resume-{all,clippy,tika,public}.log and focused resolver test result. Fix real fixture regressions; update exact expectations only for verified payload corrections. Complete full gate on cleaned head, reset checkpoint, push and merge #631 only when exact-head CI is green.
