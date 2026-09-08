---
name: milestone-executor
description: Use when executing an approved PSTD milestone or epic. Coordinate ordered issues, but implement and checkpoint one issue at a time so continuation does not require reloading the whole milestone.
---

# Milestone Executor

## Purpose

Drive an approved milestone or epic through its issue order while keeping each implementation slice independently resumable and durable.

## Inputs required

- Approved milestone or epic definition.
- Ordered issue list.
- Known scope and out-of-scope items.
- Current branch/PR state, if any.
- Known validation commands, if available.

## Execution model

1. Use the milestone/epic only to determine ordering and dependencies.
2. Select exactly one ready issue as the active implementation scope.
3. Continue an existing implementation branch/PR when present; otherwise create the dedicated branch/PR required by root `AGENTS.md`.
4. Create or update `docs/operations/active-implementation-checkpoint.md` with the active issue and `Next exact change`.
5. Implement the issue in bounded increments, committing each coherent increment before broad research, full validation, environment work, or further delegation.
6. Use focused tests during implementation and record exact results in the checkpoint.
7. When the active issue is complete, run the required merge-ready validation for that delivery boundary, update final docs, and merge if green.
8. Move to the next issue only after the previous issue has durable completion/merge evidence or the dependency model explicitly requires a shared branch.

Do not keep an entire epic's child issues, historical evidence, and source areas in active context. Do not re-read completed issue analysis when the checkpoint already records its conclusions.

## Branching rule

Prefer one branch/PR per issue or smallest coherent vertical slice. A shared milestone branch is allowed only when issues cannot safely land independently; record the coupled scope in the checkpoint. Even on a shared branch, only one issue should be active at a time and each issue boundary should have a durable commit.

## Delegation rule

Specialist roles are optional, not a default workforce fan-out. Delegate only separable questions and require durable output: a commit, durable issue/PR evidence, or a concise conclusion added to the checkpoint.

## Testing rule

During implementation, use focused validation appropriate to the changed boundary. Full CI/public-fixture validation gates the merge-ready head, not every checkpoint commit.

When a focused test cannot run, record the exact blocker and an unverified checkpoint commit rather than losing coherent implementation work. Required merge validation cannot remain deferred.

## Stop conditions

Stop and report only when:

- the active issue definition is genuinely ambiguous and no repo evidence can resolve it;
- required files or repository permissions are missing;
- the work requires secrets or production access outside authorization;
- continuing would violate the active issue's explicit scope or safety boundary.

Token/context pressure is not a stop condition: persist the current coherent increment and checkpoint before any long investigation.

## Output

Return active issue, latest durable commit, branch/PR, checkpoint state, implementation summary, focused validation, unresolved blocker, and next exact change. Report milestone-wide coverage separately and compactly.