# PSTD v1 M25 Ordered Issue Plan

## Milestone

M25: v1 Release Candidate and Operator Handoff.

## Tracking issue

- #141: M25 milestone tracking.

## Ordered implementation issues

| Order | Issue | Title | Purpose |
|---:|---:|---|---|
| 1 | #177 | [M25-I01] Create v1 release-candidate checklist and operator handoff | Add RC checklist and local/Docker operator handoff docs. |
| 2 | #178 | [M25-I02] Mark v1 roadmap and status as release-candidate complete | Update repo-wide status, roadmap, PRD, README, changelog, and docs index. |
| 3 | #179 | [M25-I03] Validate and merge M25 | Open PR, validate CI, merge, close issues, and report final v1 status. |

## Execution order

1. Add M25 milestone and implementation docs.
2. Add release-candidate checklist and operator handoff docs.
3. Add unsupported/deferred areas doc.
4. Update product, status, roadmap, validation, output contract, and changelog docs.
5. Open PR and run GitHub Actions CI.
6. Patch any CI failures.
7. Squash merge when CI is green.
8. Close #141 and #177-#179 as completed.
9. Report that M1-M25 are complete and post-v1 Snowflake ingestion planning is next.

## Validation gate

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo run -- --help
cargo run -- version
cargo run -- inspect --help
cargo run -- batch --help
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
```

For approved fixtures only:

```text
cargo run -- inspect --input <approved-small-fixture.pst>
cargo run -- extract --input <approved-small-fixture.pst> --output <tmp-output>
cargo run -- batch --input <approved-fixture-directory-or-file> --output <tmp-output>
```

## Final handoff statement

After M25 merges, the planned v1 implementation lane is complete. Any new functional implementation should be planned as post-v1 work.
