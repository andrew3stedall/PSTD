# PSTD Documentation Status

_Last reviewed: 14 July 2026._

## Purpose

Define which Markdown files represent current truth, which are point-in-time historical evidence, and how future changes must keep the repository navigable without rewriting useful history.

## Review outcome

The repository contains several generations of documentation:

1. early M1-M25 product and implementation planning;
2. PQ1-PQ74 parser-quality milestones and fixture investigations;
3. Vertical 1+ extraction milestones;
4. current architecture, operations, data-contract, onboarding, and repository instructions;
5. reusable agent skills and process references.

Older files often contain a “next milestone” or “current blocker” that was correct when written but has since been resolved. Those statements are not errors when the file is clearly treated as a point-in-time record. The error was allowing them to compete with the current README and status pages.

## Authoritative current-state files

These files must be updated whenever capability, roadmap, validation, output maturity, or operating mode changes:

| File | Authority |
|---|---|
| `README.md` | Project intent, headline progress, commands, limitations, and start-here links. |
| `AGENTS.md` | Current repository operating model and delivery rules. |
| `docs/README.md` | Documentation navigation and current/historical classification. |
| `docs/product/project-status.md` | Detailed merged capability, active work, fixture baseline, and blockers. |
| `docs/product/pstd-v1-roadmap.md` | Active extraction roadmap and deferred systems. |
| `docs/product/pstd-v1-mvp-prd.md` | Current product requirements and completion criteria. |
| `docs/operations/public-pst-progress-log.md` | End-to-end public fixture evidence over time. |
| `docs/operations/local-validation.md` | Validation gate and fixture-review procedure. |
| `docs/operations/v1-unsupported-deferred-areas.md` | Current limitations and deferred systems. |
| `docs/architecture/system-overview.md` | Current implemented architecture and boundaries. |
| `docs/engineering/codebase-map.md` | Current source-module map. |
| `docs/engineering/developer-guide.md` | Current developer workflow. |
| `docs/data/pstd-v1-output-contract-summary.md` | Current structured output contract and maturity distinctions. |
| `docs/changelog/unreleased.md` | Consolidated unreleased change history and known gaps. |
| `docs/wiki/Home.md` | Compact navigation. |
| `docs/wiki/developer-onboarding.md` | Current onboarding baseline. |
| `.agents/skills/README.md` | Skill navigation and current-mode precedence. |

If two current-state files disagree, `README.md`, `AGENTS.md`, project status, and the public progress log take precedence until the inconsistency is corrected.

## Point-in-time historical records

The following are intentionally retained as historical evidence:

### `docs/milestones/`

Completed milestone and PQ reports. They capture objective, evidence, implementation, validation, and the proposed next boundary at that time.

### `docs/issues/`

Ordered issue plans for earlier delivery phases. They are not the active backlog unless a current roadmap page explicitly reactivates them.

### `docs/engineering/*implementation-plan*.md`

Implementation plans for completed milestones. Current engineering guides in the same directory remain authoritative; plan files remain historical.

### `docs/operations/pq*.md`

Detailed PQ findings and fixture diagnostics. These are experimental records and may describe blockers that later PQ or vertical work resolved.

### `docs/operations/vertical-*.md`

Point-in-time extraction milestone records. The highest merged vertical and active PR are summarised in project status and the roadmap.

### `docs/epics/`

Early product decomposition retained for traceability.

Historical files should not be bulk rewritten to replace their original next step with the present one. Doing so would destroy the reasoning trail. Instead, keep current navigation explicit and add a correction note only when a historical file contains a factual error about what happened in that milestone.

## Decisions and research

### `docs/decisions/`

Architecture Decision Records remain valid unless superseded by a later ADR. A superseded ADR should link to its replacement rather than being rewritten silently.

### `docs/research/`

Research documents are evidence and guidance, not capability claims. Verify external facts when they materially affect implementation.

## Repository skills

Markdown under `.agents/skills/` is reusable process guidance. Older skills may use milestone/epic terminology from the completed planning lane. `AGENTS.md` and `.agents/skills/README.md` define the current interpretation: one coherent evidence-led vertical extraction milestone.

Individual role/reference skills do not need continual progress-counter edits unless their procedure or technical guidance has become incorrect.

## Duplication policy

Some early PQ documents exist in both `docs/milestones/` and `docs/operations/`. They remain historical duplicates. New vertical work should use one operations record per milestone and link it from current-state pages rather than creating multiple near-identical copies.

## Link and freshness rules

Every meaningful extraction change must:

1. update the point-in-time vertical record;
2. update project status;
3. append the public fixture result;
4. update the roadmap if the next boundary changes;
5. update the root README when headline capability changes;
6. update architecture, codebase, data, operations, or onboarding guides when their instructions change;
7. update the changelog;
8. keep links relative and valid;
9. use exact dates for reviewed current-state documents;
10. avoid dynamic claims such as “CI is green” unless tied to a specific PR/SHA or kept in the PR rather than durable docs.

## Current documentation baseline

As of this review:

- merged implementation is complete through PR #429 / Vertical 13;
- draft PR #430 is active and unmerged;
- complete recipient record assembly is validated on `main`;
- same-run projection and production publication remain incomplete;
- downstream Snowflake/UI/search work remains parked;
- historical M/PQ/vertical files are retained as traceability records and are no longer used as the current roadmap.
