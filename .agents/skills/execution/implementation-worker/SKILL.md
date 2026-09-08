---
name: implementation-worker
description: Use when writing code for one approved PSTD issue or vertical extraction slice. Resume from the active checkpoint, persist bounded increments early, and reserve full validation for merge readiness.
---

# Implementation Worker

## Purpose

Implement one scoped PSTD issue or vertical extraction slice without repeatedly reconstructing repository context.

## Startup

1. Read root `AGENTS.md`.
2. If `docs/operations/active-implementation-checkpoint.md` exists on the active branch, read it and the active issue first.
3. Inspect only the current diff, latest durable implementation commit, source files/functions named by the checkpoint, and any directly relevant failing check.
4. Continue from `Next exact change`.

Do not reread the full epic, historical parity corpus, project status, public fixture log, or unrelated source areas on a valid continuation checkpoint.

## Rules

- Work on exactly one active issue unless the user explicitly asks for a coupled implementation and the checkpoint records that scope.
- Treat a parent epic as dependency/navigation context, not active working scope.
- Continue an existing implementation branch or PR rather than duplicating work.
- Prefer small, understandable changes.
- Keep at most one bounded implementation increment uncommitted.
- After a coherent increment, make it durable before broad research, full validation, environment/toolchain work, or another delegation.
- Reuse validated components and fail closed on unsupported or ambiguous evidence.
- Add or update focused tests with the implementation.
- Update the active checkpoint whenever implementation state, durable commit, test result, blocker, or next exact change changes.
- Update current-state and compatibility documentation when behaviour changes, normally near merge readiness rather than after every WIP increment.
- Do not treat a large source file, truncated connector output, or whole-file contents API as an implementation blocker.

## Delegation

Use specialist agents only for separable questions. A delegation must produce one of:

- a durable code/test/docs commit;
- a durable issue/PR comment with reusable evidence; or
- a concise conclusion persisted into the active checkpoint.

Do not launch broad parallel fan-out for a bounded coding issue. Do not repeat a delegated investigation once its conclusion is checkpointed unless new evidence invalidates it.

## Editing method

Use this preference order:

1. Direct connector edits for small files and complete contents that can be handled safely.
2. Authenticated local `git` and `gh` checkout when available.
3. Temporary same-repository GitHub Actions checkout-and-patch workflow only when incremental large-file editing or targeted validation genuinely requires a checked-out runner.

The Actions method is not a prerequisite to making progress. Follow root `AGENTS.md` for exact-match replacements, branch guards, minimum permissions, cleanup, and runner pushes.

## Commit cadence

- Commit each coherent implementation/test increment after focused formatting/tests when readily available.
- If focused validation cannot run yet, a syntactically coherent WIP checkpoint commit is allowed; mark it unverified in the checkpoint and validate before merge.
- Never spend a long turn preparing a full portable toolchain while meaningful source changes remain only in transient context.
- Prefer several small durable commits during implementation; squash at merge.

## Validation

During implementation:

- run the smallest focused formatting/test command that covers the changed boundary;
- inspect only relevant failure output and iterate;
- do not rerun the full repository gate after every small commit.

Before merge:

- remove temporary patch/workflow scaffolding;
- run the full repository validation gate and relevant fixture workflows on the exact cleaned PR head;
- inspect the final diff and required review state;
- never claim a test passed unless its result was inspected.

Use `deferred-testing` only for a concrete blocker. Required merge validation may not remain deferred.

## Output

Return active issue, branch/PR, latest durable implementation commit, files changed, focused tests run, checkpoint state, exact observable result, fail-closed boundary, unresolved blocker, and next exact change.