# PSTD v1 M25: Release Candidate and Operator Handoff

## Goal

Close the bounded v1 implementation lane with release-candidate validation, documentation cleanup, and a clear local/Docker operator handoff.

## Final v1 position

M25 is the final planned v1 milestone. After M25 is merged, the planned v1 milestone lane is complete across M1-M25.

## M25 scope

| Area | M25 deliverable | Outcome |
|---|---|---|
| Release-candidate checklist | Add an explicit checklist for CI, local validation, fixture validation, and handoff readiness. | Operators know what must pass before treating v1 as ready for local/Docker use. |
| Operator handoff | Add local and Docker runbooks. | A new operator can validate, inspect, extract, and batch process approved inputs. |
| Unsupported/deferred areas | Document non-blocking limitations and post-v1 boundaries. | Deferred work is explicit and does not blur v1 scope. |
| Roadmap/status cleanup | Update README, project status, PRD, roadmap, changelog, and docs index. | Repo-level status reflects that M25 completes the planned v1 milestone lane. |
| Validation tracking | Open PR, run CI, patch failures, squash merge if green. | M25 closes with CI evidence. |

## Release-candidate meaning

PSTD v1 release-candidate status means:

- the Rust CLI, Python wrapper, Docker build, CLI smoke checks, and available fixture smoke checks pass CI;
- repo docs define how to run and validate local/Docker extraction;
- outputs are documented for single-PST and batch extraction;
- unsupported and deferred areas are documented;
- post-v1 planning starts with Snowflake ingestion planning, not implementation.

It does not mean every PST layout is fully decoded. Parser quality still depends on broader fixture coverage and future observed-layout work.

## Out of scope

- Snowflake implementation.
- Search, semantic search, embeddings, graph, tagging, or web UI implementation.
- Distributed orchestration.
- Outlook automation or third-party PST parser integration.
- Audit-grade exact-preservation archive mode.

## Acceptance criteria

- v1 release-candidate status is documented in the repo.
- Operator commands and validation expectations are clear.
- Remaining unsupported/deferred areas are tracked without blocking v1 release-candidate status.
- CI passes before merge.

## Handoff after merge

After M25 merges, the next planning lane is post-v1 Snowflake ingestion planning.
