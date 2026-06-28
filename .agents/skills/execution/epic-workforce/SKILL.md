---
name: epic-workforce
description: Use when coordinating several PSTD role skills to build an approved epic or milestone in a dependency-aware order.
---

# Epic Workforce

## Purpose

Coordinate the specialist skills needed to build a complete epic or milestone without prompting for every single issue.

## Workforce roles

Use these role skills as needed:

- `roles/product` for product scope checks.
- `roles/business-analyst` for issue order and requirements.
- `roles/ux` for CLI, API, and user flow details.
- `roles/data` for output contracts and data shape.
- `roles/platform` for runtime and validation notes.
- `roles/full-stack-developer` for implementation planning.
- `roles/integration` for combining changes.
- `roles/reviewer` for final review.
- `docs-writer` for documentation updates.

## Working rules

- Follow the approved milestone issue order.
- Keep work on the milestone branch.
- Make coherent commits or file updates that match the milestone scope.
- Do not wait for a new prompt after every issue if the next issue is defined by the milestone.
- Pause only for unclear scope, risky access, or a missing product decision.

## Output

Return issue coverage, role notes, implementation summary, integration notes, deferred tests, and review checklist.
