# Phone-First Operating Model

## Purpose

PSTD planning and implementation can be run from ChatGPT on mobile through the GitHub connector while preserving enough durable state to resume long-running work cheaply.

## Current approach

```text
ChatGPT mobile prompt
  -> GitHub connector
  -> root AGENTS.md
  -> branch-local active checkpoint
  -> one active issue
  -> bounded implementation commit
  -> focused validation
  -> checkpoint update
  -> repeat
  -> exact-head merge validation
```

## Why this approach

This keeps the project moving without requiring a local Codex install and prevents each continuation from paying the full repository-discovery cost again.

## What works well

- Planning reports when needed.
- Developer-ready issue design.
- Documentation updates.
- Branch and PR creation.
- Direct repository edits supported by the connector.
- Long-running implementation when state is persisted in small commits and an active checkpoint.
- Targeted GitHub Actions runners for large-file editing or focused validation when genuinely required.

## Continuation loop

When an implementation branch/PR exists:

1. Read `AGENTS.md`, `docs/operations/active-implementation-checkpoint.md`, and the active issue.
2. Inspect the current diff/latest durable implementation commit and only source areas named by the checkpoint.
3. Continue from `Next exact change`.
4. Run focused validation when readily available.
5. Commit the coherent increment before broad research, environment setup, full validation, or another delegation.
6. Update the checkpoint.
7. Repeat until implementation is complete.
8. Run full exact-head validation/fixtures only when preparing to merge.

Do not bulk-read current-state history, the full parent epic, or historical parity/PQ/milestone documents on a valid continuation.

## Fresh-start loop

1. Read the minimum current-state context required by `AGENTS.md`.
2. Check existing PRs/branches to avoid duplicate work.
3. Select exactly one active issue or smallest coherent vertical slice.
4. Create the branch and draft PR.
5. Create the active implementation checkpoint immediately.
6. Switch to the continuation loop.

## Delegation

Parallel specialist fan-out is not the default. Delegate only a separable question whose result can be made durable as a commit, issue/PR evidence, or checkpoint conclusion. Persist the result before launching overlapping follow-up work.

## Validation model

Focused validation supports iteration. Full repository validation and approved fixture evidence gate merge readiness, not intermediate checkpoint commits. A coherent increment may be committed as explicitly unverified if focused validation is temporarily unavailable; it must be verified before merge.

## Limits

This mode does not grant arbitrary repository administration, secret access, paid infrastructure changes, unrestricted deployment access, or true background execution. Repository permissions and safety boundaries still apply.

## Repo skills

The reusable skill files live under `.agents/skills/`. Root `AGENTS.md`, the active checkpoint, and the active issue take precedence during continuation work.
