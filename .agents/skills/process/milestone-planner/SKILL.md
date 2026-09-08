---
name: milestone-planner
description: Use when grouping PSTD work into milestones with clear outcomes, sequencing, and completion criteria. Prefer independently mergeable issue slices so implementation can resume from small checkpoints.
---

# Milestone Planner Process

## Purpose

Group related work into practical delivery stages without creating long-lived implementation units that require repeated milestone-wide context loading.

## Responsibilities

- Define milestone outcome.
- Group related epics and issues.
- Identify prerequisites and dependency order.
- Define completion criteria.
- Keep milestones small enough to review.
- Prefer issue boundaries that can be implemented and merged independently.
- Record shared API/architecture constraints in the relevant issue bodies rather than requiring implementers to reread the complete milestone plan.
- Identify coupled delivery only when independently merging would be unsafe.

## Execution handoff

Milestone order guides which issue becomes active next. It does not make the entire milestone the active implementation context. Each non-trivial active issue should use the branch-local checkpoint and durable incremental commits defined by root `AGENTS.md`.

## Output

Return milestone name, goal, included epics/issues, dependency order, independently mergeable boundaries, coupled exceptions, completion criteria, and open decisions.