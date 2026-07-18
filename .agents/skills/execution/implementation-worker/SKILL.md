---
name: implementation-worker
description: Use when writing code for an approved PSTD vertical milestone or epic. Prefer validated implementation through a local checkout or the temporary GitHub Actions checkout-and-patch method.
---

# Implementation Worker

## Purpose

Implement one scoped PSTD vertical milestone from approved current-state evidence.

## Rules

- Work only inside the approved milestone or epic scope.
- Continue an existing implementation branch or PR rather than duplicating work.
- Prefer small, understandable changes.
- Do not rewrite unrelated areas.
- Reuse validated components and fail closed on unsupported or ambiguous evidence.
- Add or update focused tests with the implementation.
- Update current-state and compatibility documentation when behaviour changes.
- Do not treat a large source file, truncated connector output, or whole-file contents API as an implementation blocker.

## Editing method

Use this preference order:

1. Direct connector edits for small files and complete contents that can be handled safely.
2. Authenticated local `git` and `gh` checkout when available.
3. Temporary same-repository GitHub Actions checkout-and-patch workflow for incremental changes to large existing files.

For the Actions method, follow root `AGENTS.md`: exact-match replacements, same-repository branch guards, minimum permissions, no secrets, focused validation before commit, runner pushes limited to intended non-workflow files, connector-managed workflow changes and cleanup, and final exact-head diff inspection.

## Validation

- Run formatting and focused tests before committing through the temporary runner.
- Run the full repository validation gate and relevant fixture workflows on the cleaned PR head.
- Use `deferred-testing` only for a concrete blocker that prevents local, temporary-runner, and CI validation.
- Never claim a test passed unless its result was inspected.

## Output

Return milestone scope, branch and PR, files changed, editing method used, tests added and run, exact observable result, fail-closed boundary, cleanup status, unresolved blocker, and next validation or implementation step.