# PSTD Wiki Home

_Last reviewed: 8 September 2026._

This folder provides a compact repo-hosted navigation layer. Current product truth lives in the maintained current-state pages; in-progress implementation state lives in the branch-local active checkpoint.

## Main pages

| Need | Page |
|---|---|
| Resume active implementation | `AGENTS.md` + branch-local `docs/operations/active-implementation-checkpoint.md` + active issue |
| Checkpoint format | [Implementation Checkpoint Template](../operations/implementation-checkpoint-template.md) |
| Project intent and headline progress | [Root README](../../README.md) |
| Current merged capability and blocker | [Project Status](../product/project-status.md) |
| Real-fixture evidence | [Public PST Progress Log](../operations/public-pst-progress-log.md) |
| Active extraction roadmap | [Roadmap](../product/pstd-v1-roadmap.md) |
| Developer onboarding | [Developer Onboarding](developer-onboarding.md) |
| Code structure | [Codebase Map](../engineering/codebase-map.md) |
| System architecture | [System Overview](../architecture/system-overview.md) |
| Output contract | [Output Contract Summary](../data/pstd-v1-output-contract-summary.md) |
| Validation | [Local Validation](../operations/local-validation.md) |
| Documentation/context policy | [Documentation Status](../DOCUMENTATION_STATUS.md) |

## Working rule

### Continuation

Read `AGENTS.md`, the active checkpoint, and the active issue. Inspect the current diff/latest durable commit and only the source boundary or failing check named by the checkpoint. Continue from `Next exact change`.

Do not bulk-read current-state history, the parent epic, PQ/vertical records, or the readpst-gap corpus on a valid continuation.

### Fresh start

Read the minimum current-state pages required by `AGENTS.md`, check existing PRs/branches, select one issue/slice, create its branch/PR/checkpoint, then use the continuation path.

Current delivery counters and active PRs change frequently; use Project Status/GitHub rather than duplicating them here.
