---
name: milestone-branch-manager
description: Use when planning branch structure for a PSTD milestone or epic and preparing a milestone PR.
---

# Milestone Branch Manager

## Purpose

Keep epic and milestone work organised in branches and PRs.

## Branch pattern

Use one milestone branch for a coherent milestone or epic:

- `milestone/<short-name>` for milestone delivery.
- `epic/<short-name>` for a narrower epic.

## Rules

- Branch from current `main` unless directed otherwise.
- Keep the branch aligned to one approved milestone or epic.
- Do not mix unrelated milestones.
- Use the milestone issue order to guide implementation.
- Open one PR for the milestone branch.
- Include a checklist of issues covered.
- Include deferred local test commands.

## Output

Return branch name, covered issues, omitted issues, PR summary, test notes, and merge readiness notes.
