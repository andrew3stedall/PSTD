# Planning Labels

Create these GitHub labels before relying on automated planning.

## Workflow

- `codex:intake`: raw idea or PRD requiring planning.
- `codex:planning`: planning is underway.
- `codex:ready`: ready for implementation.
- `codex:blocked`: blocked by a missing decision or dependency.
- `codex:needs-feedback`: human clarification required.
- `codex:manual`: human-only work.
- `codex:review`: planning output or PR requires review.

## Roles

- `role:product`: product value or scope.
- `role:ba`: requirements and issue design.
- `role:ux`: user experience and interaction design.
- `role:dev`: implementation work.
- `role:data-science`: analysis, inference, anomaly, or evaluation work.
- `role:data-engineering`: data movement, modelling, contracts, or scale.
- `role:systems`: infrastructure, operational, and delivery concerns.
- `role:docs`: documentation work.

## Risk

- `risk:low`: isolated low-risk work.
- `risk:medium`: normal review required.
- `risk:high`: explicit human approval required before implementation.
- `risk:data`: data correctness, privacy, volume, or retention concern.
- `risk:migration`: schema or irreversible data migration concern.
- `risk:deployment`: release or environment concern.

## Parallelism

Parallel execution is deliberately delayed for PSTD.

- `parallel:safe`: can run alongside other tasks.
- `parallel:isolated`: can run in parallel if files do not overlap.
- `parallel:serial`: must be sequenced alone.
- `parallel:blocked`: cannot run until dependency is resolved.

## Priority

- `priority:high`
- `priority:medium`
- `priority:low`
