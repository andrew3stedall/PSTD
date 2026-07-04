# PSTD v1 M22 Ordered Issue Plan

## Milestone

M22: Body and Header Fidelity Expansion.

## Tracking issue

- #137: M22 milestone tracking.

## Ordered implementation issues

| Order | Issue | Title | Purpose |
|---:|---:|---|---|
| 1 | #161 | [M22-I01] Add Unicode HTML body extraction | Support reachable Unicode/string HTML body properties. |
| 2 | #162 | [M22-I02] Surface transport headers on message records | Add transport headers to message JSONL records when available. |
| 3 | #163 | [M22-I03] Update M22 docs and validation handoff | Link M22 docs and record M23 handoff. |

## Execution order

1. Add the selected Unicode HTML MAPI property and extraction path.
2. Add transport headers to `MessageRecord` constructors.
3. Add focused tests for body and header transitions.
4. Update output-contract and milestone documentation.
5. Open a PR and run GitHub Actions CI.
6. Squash merge if CI passes, then close #137 and #161-#163.

## Validation gate

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo run -- --help
cargo run -- batch --help
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
```

## Handoff

M22 should hand off to M23 once CI is green. M23 should focus on attachment payload fidelity and should preserve the M20-M22 compact decoder, body, and header contracts.
