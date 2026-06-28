# Planning Labels

Create these GitHub labels before relying on structured planning.

Some original label names could not be created through the ChatGPT GitHub connector because the connector blocked specific words. This document uses the safer label names that can be managed more reliably from a phone-first workflow.

## Workflow

- `codex:intake`: raw idea or PRD requiring planning.
- `codex:planning`: planning is underway.
- `codex:ready`: ready for implementation.
- `codex:on-hold`: paused by a missing decision or dependency.
- `codex:needs-feedback`: human clarification required.
- `codex:manual`: human-only work.
- `codex:review`: planning output or PR requires review.

## Roles

- `role:product`: product value or scope.
- `role:analysis`: requirements and issue design.
- `role:ux`: user experience and interaction design.
- `role:dev`: implementation work.
- `role:analytics`: analysis, inference, anomaly, or evaluation work.
- `role:data-engineering`: data movement, modelling, contracts, or scale.
- `role:systems`: infrastructure, operational, and delivery concerns.
- `role:documentation`: documentation work.

## Risk

- `risk:minor`: isolated low-risk work.
- `risk:medium`: normal review required.
- `risk:high`: explicit human approval required before implementation.
- `risk:data`: data correctness, privacy, volume, or retention concern.
- `risk:migration`: schema or irreversible data migration concern.
- `risk:deployment`: release or environment concern.

## Parallelism

Parallel execution is deliberately delayed for PSTD.

- `parallel:ok`: can run alongside other tasks.
- `parallel:separate`: can run in parallel if files do not overlap.
- `parallel:serial`: must be sequenced alone.
- `parallel:on-hold`: cannot run until dependency is resolved.

## Priority

- `priority:high`
- `priority:normal`
- `priority:minor`

## Original blocked names

The original taxonomy used these names, but they were blocked through the connector during setup:

- `codex:blocked`
- `role:ba`
- `role:data-science`
- `role:docs`
- `risk:low`
- `parallel:safe`
- `parallel:isolated`
- `parallel:blocked`
- `priority:medium`
- `priority:low`
