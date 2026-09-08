---
name: epic
description: Use when turning PSTD roadmap items into epics with issue breakdowns, scope, dependencies, and success criteria. Design issues so they can be executed and checkpointed independently.
---

# Epic Process

## Purpose

Turn a product goal into a delivery-ready dependency map without requiring the whole epic to remain active during implementation.

## Responsibilities

- Define the epic outcome and explicit exclusions.
- Break work into ordered, independently understandable GitHub issues.
- Make dependencies and shared API decisions explicit in issue bodies so implementation agents do not need to reread the whole epic.
- Prefer issue boundaries that can be merged independently.
- Identify the rare cases that genuinely require a shared branch or coupled delivery.
- Define success criteria and final epic-level acceptance separately from per-issue merge gates.
- Avoid requiring broad specialist fan-out when one issue can be implemented directly from durable evidence.

## Execution handoff

The epic is an ordering/dependency map. During implementation:

- activate one issue at a time;
- create/use the branch-local active checkpoint;
- persist coherent implementation increments early;
- load sibling issue details only for a concrete dependency decision.

## Output

Return epic title, outcome, scope, ordered issue breakdown, dependency order, independently mergeable boundaries, coupled exceptions, risks, and docs required.