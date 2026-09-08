---
name: milestone-branch-manager
description: Use when planning PSTD branch structure for a milestone or epic. Prefer issue-sized branches and resumable checkpoints over long-lived branches carrying an entire epic in active context.
---

# Milestone Branch Manager

## Purpose

Keep milestone and epic delivery organised while ensuring implementation can resume cheaply from durable Git state.

## Branch pattern

Default to the smallest independently mergeable delivery unit:

- `agent/<issue-or-slice>` for one implementation issue/vertical slice;
- `milestone/<short-name>` only when several issues cannot safely land independently;
- `epic/<short-name>` only for genuinely inseparable epic-wide integration.

## Rules

- Branch from current `main` unless continuing an existing branch or explicitly directed otherwise.
- Reuse an existing implementation branch/PR instead of creating a competing one.
- Keep exactly one issue active at a time even on a shared milestone branch.
- Maintain `docs/operations/active-implementation-checkpoint.md` on every non-trivial active implementation branch.
- Make a durable commit at each coherent implementation increment and at each issue boundary.
- Parent milestone/epic definitions guide ordering; they do not need to remain loaded during issue implementation.
- Open the draft PR early so branch state, commits, checkpoint, CI failures, and review notes are durable.
- Do not defer the first PR until the full milestone body is implemented.
- Keep unrelated work out of the branch.
- Remove/reset the active checkpoint when the delivery is completed so it cannot become stale current-state documentation.

## Merge rule

- Full exact-head validation and required fixture evidence gate merge.
- Intermediate commits may be explicitly unverified when focused validation is temporarily unavailable.
- Prefer squash merge so frequent checkpoint commits do not degrade main history.

## Output

Return branch name, active issue, latest durable commit, checkpoint status, covered/omitted issues, PR summary, test notes, and merge readiness.