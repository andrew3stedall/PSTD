# PSTD v1 M21 Ordered Issue Plan

## Milestone

M21: Focused Decoder Evidence Expansion.

## Tracking issue

- #136: M21 milestone tracking.

## Ordered implementation issues

| Order | Issue | Title | Purpose |
|---:|---:|---|---|
| 1 | #143 | [M21-I01] Classify UTF16 compact decoder evidence | Count M20 `CATW` status families as fixture-backed decoder evidence. |
| 2 | #144 | [M21-I02] Add focused decoder evidence tests | Prove UTF-16 compact evidence and combined compact evidence classification. |
| 3 | #145 | [M21-I03] Update M21 docs and handoff notes | Link M21 docs and record M22 handoff. |

## Execution order

1. Add the narrow evidence classifier change.
2. Add tests proving `CATW` evidence is supported and counted separately from `CATB` evidence.
3. Update milestone, implementation, roadmap, project status, changelog, and docs index.
4. Open a PR and run GitHub Actions CI.
5. Close #143-#145 and #136 after the PR is merged.

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

M21 should hand off to M22 once CI is green because the remaining parser evidence task is now to preserve the supported `CATB` and `CATW` paths while body/header fidelity is expanded.
