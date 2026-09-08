# PSTD Skills Index

_Last reviewed: 8 September 2026._

This folder contains reusable planning, implementation, review, documentation, data, and platform instructions for PSTD.

## Precedence

When instructions conflict, use this order:

1. explicit user request;
2. root `AGENTS.md`;
3. active branch checkpoint (`docs/operations/active-implementation-checkpoint.md`) and active issue for continuation work;
4. current project status, roadmap, and public fixture log for fresh-start/current-truth questions;
5. the relevant skill;
6. historical milestone, PQ, parity-gap, issue-plan, or implementation-plan documents.

Some skills were written during the completed M1-M25 milestone-planning lane. Their techniques remain useful, but current work uses evidence-led vertical extraction milestones and checkpoint-first continuation.

## Current mode

`vertical-extraction`

### When continuing existing implementation

- read root `AGENTS.md`, the active checkpoint, and the active issue;
- inspect the current diff/latest durable implementation commit and only the source areas named by the checkpoint;
- continue from `Next exact change`;
- keep one issue active at a time;
- make every coherent implementation increment durable before broad research, full validation, environment work, or further delegation;
- update the checkpoint after material changes;
- run focused tests during implementation and reserve the full CI/public-fixture gate for merge readiness.

Do **not** bulk-read the full epic, project history, PQ records, or `docs/readpst-gaps/` corpus on a valid continuation. Historical files should be fetched only when the current issue/checkpoint points to a specific evidence need.

### When starting new implementation

- inspect open PRs/branches to avoid conflicting work;
- read the minimum current-state sources required by root `AGENTS.md` once;
- identify exactly one highest-value extraction issue or coherent slice;
- create the branch/PR and active checkpoint immediately;
- then switch to the continuation workflow.

Local testing must not be claimed when it was not run. A phone or connector workflow is not automatically a testing blocker. Full exact-head CI must pass before merge, but it does not gate intermediate checkpoint commits.

## References and assets

- [References and assets index](references-and-assets.md)

Use repository references before inventing output formats, CLI behaviour, property semantics, diagnostic shapes, or fixture claims.

## Core skills

- `planning-council`: structured multi-role planning when a genuinely new product/architecture decision is required.
- `issue-writer`: developer-ready issue bodies.
- `docs-writer`: current-state and point-in-time documentation.
- `github-planning-loop`: mobile/connector implementation loop with checkpoint-first continuation.

## Role skills

- `roles/executive-sponsor`: alignment and scope control.
- `roles/product`: product value and extraction priority.
- `roles/business-analyst`: requirements and acceptance boundaries.
- `roles/ux`: CLI, API, and developer experience.
- `roles/developer-feasibility`: implementation feasibility.
- `roles/full-stack-developer`: implementation planning; downstream stacks remain parked unless explicitly activated.
- `roles/metrics`: fixture measurements and progress signals.
- `roles/data`: output contracts and completeness states.
- `roles/platform`: validation, CI, Docker, and operating constraints.
- `roles/integration`: sequencing and overlap checks.
- `roles/reviewer`: correctness, scope, and readiness review.

## Process skills

- `process/prd-intake`
- `process/milestone-planner`
- `process/epic`
- `process/dependency-mapper`
- `process/risk-reviewer`
- `process/readiness-check`
- `process/feedback-refiner`

Use these when the work genuinely requires planning. Do not force a planning council, epic-wide reread, or role fan-out over a bounded, already evidenced implementation issue.

## Execution skills

- `execution/milestone-executor`
- `execution/epic-workforce`
- `execution/implementation-worker`
- `execution/milestone-branch-manager`
- `execution/deferred-testing`

Treat “milestone” in older skill names as a scoped delivery unit. For current parser work, the implementation unit should normally be one GitHub issue or one coherent vertical extraction slice, with the parent epic used for ordering and dependencies rather than as active context.