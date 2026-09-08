# PSTD Documentation Status

_Last reviewed: 8 September 2026._

## Purpose

Define which Markdown files represent current truth, which are temporary execution state, which are point-in-time historical evidence, and how future changes must keep the repository navigable without forcing agents to reread project history on every continuation.

## Review outcome

The repository contains several generations of documentation:

1. early M1-M25 product and implementation planning;
2. PQ1-PQ74 parser-quality milestones and fixture investigations;
3. Vertical extraction milestone records;
4. readpst parity/gap evidence and implementation plans;
5. current architecture, operations, data-contract, onboarding, and repository instructions;
6. reusable agent skills and process references;
7. temporary branch-local implementation checkpoints.

Older files often contain a “next milestone” or “current blocker” that was correct when written but has since been resolved. Those statements are not errors when the file is clearly treated as a point-in-time record. They must not compete with current-state documents or become default continuation context.

## Authoritative current-state files

These files are durable sources of current truth and should be updated when their subject materially changes:

| File | Authority |
|---|---|
| `README.md` | Project intent, headline progress, commands, limitations, and start-here links. |
| `AGENTS.md` | Current repository operating model, continuation fast path, and delivery rules. |
| `docs/README.md` | Documentation navigation and current/historical classification. |
| `docs/product/project-status.md` | Detailed merged capability, active work, fixture baseline, and blockers. |
| `docs/product/pstd-v1-roadmap.md` | Active extraction roadmap and deferred systems. |
| `docs/product/pstd-v1-mvp-prd.md` | Current product requirements and completion criteria. |
| `docs/operations/public-pst-progress-log.md` | End-to-end public fixture evidence over time. |
| `docs/operations/local-validation.md` | Validation gate and fixture-review procedure. |
| `docs/operations/implementation-checkpoint-template.md` | Required compact format for branch-local implementation handoffs. |
| `docs/operations/v1-unsupported-deferred-areas.md` | Current limitations and deferred systems. |
| `docs/architecture/system-overview.md` | Current implemented architecture and boundaries. |
| `docs/engineering/codebase-map.md` | Current source-module map. |
| `docs/engineering/developer-guide.md` | Current fresh-start and continuation developer workflow. |
| `docs/data/pstd-v1-output-contract-summary.md` | Current structured output contract and maturity distinctions. |
| `docs/readpst-gaps/README.md` | Durable readpst capability comparison and parity/gap index; not default continuation startup context. |
| `docs/changelog/unreleased.md` | Consolidated unreleased change history and known gaps. |
| `docs/wiki/Home.md` | Compact navigation. |
| `docs/wiki/developer-onboarding.md` | Current onboarding and continuation entry point. |
| `.agents/skills/README.md` | Skill navigation and current-mode precedence. |

If current-state files disagree, `README.md`, `AGENTS.md`, project status, and the public progress log take precedence until the inconsistency is corrected. For an in-progress branch, the active checkpoint and active issue define execution state but do not override durable product truth.

## Temporary branch-local execution state

### `docs/operations/active-implementation-checkpoint.md`

This file exists only on a non-trivial active implementation branch/PR. It records:

- active issue/scope;
- latest durable implementation commit;
- established conclusions that should not be re-derived;
- current source boundary;
- focused validation state;
- blockers;
- next exact change.

It is deliberately concise and is the first continuation handoff after `AGENTS.md`. It should normally remain under roughly 100 lines.

The active checkpoint is **not** durable product documentation and should not be left stale on `main`. Delete/reset it when its implementation PR is completed. Use `docs/operations/implementation-checkpoint-template.md` to create it.

## Point-in-time historical records

The following are intentionally retained as historical evidence and are not default startup/continuation reads.

### `docs/milestones/`

Completed milestone and PQ reports. They capture objective, evidence, implementation, validation, and the proposed next boundary at that time.

### `docs/operations/perf-01-content-output.md`

Point-in-time output-scaling measurements and validation for issue #595. Current performance/capability claims belong in the maintained current-state docs.

### `docs/issues/`

Ordered issue plans for earlier delivery phases. They are not the active backlog unless a current roadmap or GitHub issue explicitly reactivates them.

### `docs/engineering/*implementation-plan*.md`

Implementation plans for completed milestones. Current engineering guides in the same directory remain authoritative; plan files remain historical.

### `docs/operations/pq*.md`

Detailed PQ findings and fixture diagnostics. These are experimental records and may describe blockers that later PQ or vertical work resolved.

### `docs/operations/vertical-*.md`

Point-in-time extraction milestone records. Current merged capability belongs in project status/current roadmap.

### `docs/epics/`

Early product decomposition retained for traceability.

### `docs/readpst-gaps/00-*.md` through `14-*.md`

Durable parity research, source anchors, matrices, plans, and orchestration records. They may be consulted for a specific active issue, but agents must not bulk-read this corpus on routine continuation. The index can point to the exact file/section required.

Historical files should not be bulk rewritten to replace their original next step with the present one. Doing so would destroy the reasoning trail. Add a correction note only when a historical file contains a factual error about what happened at that time.

## Decisions and research

### `docs/decisions/`

Architecture Decision Records remain valid unless superseded by a later ADR. A superseded ADR should link to its replacement rather than being rewritten silently.

### `docs/research/`

Research documents are evidence and guidance, not capability claims. Verify external facts when they materially affect implementation.

## Repository skills

Markdown under `.agents/skills/` is reusable process guidance. `AGENTS.md`, `.agents/skills/README.md`, and the active checkpoint define current continuation behavior.

Older skills may use milestone/epic terminology from the completed planning lane. Interpret that as a delivery grouping, not a requirement to keep an entire epic in working context. Current implementation should normally activate one GitHub issue or smallest coherent vertical slice at a time.

## Context-loading policy

### Fresh start

Read the minimum current-state documents required by `AGENTS.md`, check active branches/PRs, select one issue, create the branch/PR/checkpoint, then switch to the continuation path.

### Continuation

Read only:

1. `AGENTS.md`;
2. active checkpoint;
3. active issue;
4. current diff/latest durable implementation commit;
5. source files/functions and failing checks named by the checkpoint.

Do not reread project history, full epics, PQ records, parity-gap corpus, or unrelated CI unless new evidence makes it relevant.

## Duplication policy

Some early PQ documents exist in both `docs/milestones/` and `docs/operations/`. They remain historical duplicates. New work should avoid creating another narrative copy when an issue, commit, checkpoint, or existing current-state page can hold the information.

## Link and freshness rules

Every meaningful extraction change must, before merge where applicable:

1. update the point-in-time vertical record when useful;
2. update project status;
3. append the public fixture result;
4. update the roadmap if the next boundary changes;
5. update the root README when headline capability changes;
6. update architecture, codebase, data, operations, or onboarding guides when their instructions change;
7. update the changelog;
8. keep links relative and valid;
9. use exact dates for reviewed current-state documents;
10. avoid dynamic claims such as “CI is green” unless tied to a specific PR/SHA or kept in the PR/checkpoint rather than durable docs.

During WIP implementation, do not repeatedly update every current-state document after each checkpoint commit. Keep execution state in the checkpoint and perform final durable documentation updates at the merge-ready boundary.
