# PSTD Skills Index

_Last reviewed: 18 July 2026._

This folder contains reusable planning, implementation, review, documentation, data, and platform instructions for PSTD.

## Precedence

When instructions conflict, use this order:

1. explicit user request;
2. root `AGENTS.md`;
3. current project status, roadmap, and public fixture log;
4. the relevant skill;
5. historical milestone, PQ, issue-plan, or implementation-plan documents.

Some skills were written during the completed M1-M25 milestone-planning lane. Their techniques remain useful, but current work uses evidence-led vertical extraction milestones.

## Current mode

`vertical-extraction`

Before implementing:

- inspect open PRs, active branches, recent commits, and CI;
- continue existing work rather than creating a conflicting branch;
- identify the highest-value extraction gap from current fixture evidence;
- complete one coherent vertical slice;
- fail closed and reuse validated components;
- use the root `AGENTS.md` GitHub connector implementation method when a large existing file requires an incremental edit and no usable local checkout exists;
- rerun the full CI and public fixture workflow;
- update current-state and point-in-time documentation.

Local testing must not be claimed when it was not run. A phone or connector workflow is not automatically a testing blocker: use the temporary same-repository Actions checkout-and-patch method and exact-head CI when available. CI must pass on the exact cleaned head before merge.

## References and assets

- [References and assets index](references-and-assets.md)

Use repository references before inventing output formats, CLI behaviour, property semantics, diagnostic shapes, or fixture claims.

## Core skills

- `planning-council`: structured multi-role planning when a genuinely new product/architecture decision is required.
- `issue-writer`: developer-ready issue bodies.
- `docs-writer`: current-state and point-in-time documentation.
- `github-planning-loop`: mobile/connector repository workflow, including the preferred temporary Actions checkout-and-patch method for large files.

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

Use these when the work requires planning. Do not force a planning council over a small, already evidenced vertical extraction change.

## Execution skills

- `execution/milestone-executor`
- `execution/epic-workforce`
- `execution/implementation-worker`
- `execution/milestone-branch-manager`
- `execution/deferred-testing`

Treat “milestone” in older skill names as a scoped delivery unit. For current parser work, that unit should be one coherent vertical extraction milestone.