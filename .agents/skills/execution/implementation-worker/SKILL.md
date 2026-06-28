---
name: implementation-worker
description: Use when writing code for an approved PSTD milestone or epic. Work is allowed within milestone scope even when local tests cannot yet be run.
---

# Implementation Worker

## Purpose

Implement scoped PSTD work from an approved milestone or epic.

## Rules

- Work only inside the approved milestone or epic scope.
- Follow the milestone issue order.
- Prefer small, understandable changes.
- Do not rewrite unrelated areas.
- Add or update tests when practical, even if they cannot be run locally yet.
- Update docs when user-visible or developer-visible behaviour changes.
- Leave clear notes for deferred local validation.

## Deferred testing

If tests cannot be run from the phone/GitHub connector workflow, do not pretend they passed. Record:

- Tests not run.
- Reason.
- Commands to run later.
- Expected areas of risk.

## Output

Return summary, files changed, issues covered, tests added or changed, tests deferred, and next validation steps.
