# PQ Cycle SOP

## Purpose

Use this SOP for repeated PST quality milestones while the project is focused on absolute PST conversion coverage.

## Standard request

Use this prompt to repeat the cycle:

```text
Run the next 3-round PQ cycle from the current milestone. Complete the current PQ, inspect the public PST artifact, revise the next PQ requirements from the measured blocker, implement the next PQ, inspect again, revise the following PQ, implement it, and report what is next.
```

For a shorter version:

```text
Run the next PQ 3-cycle.
```

## Execution rules

For each PQ in the cycle:

1. Confirm there is no existing open issue or PR for the target PQ.
2. Create a compact issue only when needed.
3. Branch from `main`.
4. Implement the smallest safe change that advances PST conversion coverage or exposes the next blocker.
5. Update milestone and operations docs.
6. Run CI.
7. Inspect the `public-pst-progress` artifact.
8. Update `docs/operations/public-pst-progress-log.md`.
9. Run final-head CI.
10. Squash merge only if final-head CI is green.
11. Revise the next PQ requirements from the artifact, not from assumptions.

## Scope rule

Do not broaden raw NBT heuristics while the table-led path is blocked. Keep each milestone aligned to the latest measured blocker.
