---
name: epic-workforce
description: Use when coordinating PSTD work across an approved epic. Keep the epic as an ordering map, activate one issue at a time, and use specialists only when their output can be made durable.
---

# Epic Workforce

## Purpose

Coordinate a multi-issue epic without loading or re-deriving the entire epic on every implementation turn.

## Workforce roles

Use specialist roles only when the active issue has a separable question that benefits from them:

- `roles/product` for unresolved product scope.
- `roles/business-analyst` for ambiguous acceptance/dependency boundaries.
- `roles/ux` for CLI/API/user-flow decisions.
- `roles/data` for output contracts and data shape.
- `roles/platform` for runtime or validation blockers.
- `roles/full-stack-developer` for a specific implementation design question.
- `roles/integration` for a concrete cross-issue interface.
- `roles/reviewer` for merge-ready review.
- `docs-writer` for final/current-state documentation.

Do not invoke all roles by default.

## Model routing

Apply `execution/subagent-model-routing` whenever Codex delegates a specialist task.

- Prefer GPT-5.6 Luna for bounded low-complexity leaf tasks.
- Use GPT-5.6 Terra for ordinary moderate-reasoning implementation work.
- Reserve GPT-5.6 Sol or GPT-6 Astra for ambiguous semantics, consequential architecture/integration decisions, hard debugging, or merge-critical review.
- Do not inherit the orchestrator's stronger model merely because it is available.
- Escalate when the delegated task grows beyond its original complexity or correctness risk warrants it.

## Working rules

- Follow the approved epic issue order and dependencies.
- Select exactly one ready issue as the active implementation scope.
- Keep the parent epic as a compact ordering map; do not keep all child issue bodies and source areas in working context.
- Continue the current branch/PR/checkpoint when one exists.
- Make coherent implementation commits early and update the active checkpoint.
- Do not wait for a new prompt after every completed issue when the next issue is already defined; after merge, select the next ready issue and create its new checkpointed delivery branch.
- Use a shared epic branch only when issues are technically inseparable; document that exception in the checkpoint.

## Delegation budget

A specialist delegation must produce durable reusable output before another overlapping delegation is launched. Acceptable outputs are:

- a code/test/docs commit;
- a durable issue/PR comment containing the evidence;
- a concise conclusion added to `docs/operations/active-implementation-checkpoint.md`.

If a specialist returns no durable result, do not repeat the same broad investigation. Narrow the question or proceed from repository evidence directly.

Avoid parallel fan-out over the same source boundary. It increases context cost and creates duplicated discovery without durable progress.

## Validation

Use focused tests while an issue is under implementation. Full exact-head validation and relevant fixtures gate that issue/slice's merge. Do not run an epic-wide validation ceremony after every small increment.

## Output

Return compact epic progress, active issue, latest durable commit, checkpoint state, any specialist conclusion that was persisted, validation state, and next exact change.