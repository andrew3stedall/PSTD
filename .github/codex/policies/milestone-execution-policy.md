# Milestone Execution Policy

## Purpose

Define how PSTD can move from planning into coding without requiring the user to prompt for every issue.

## Current execution model

The approved execution unit is a milestone or epic, not a single issue.

A milestone or epic must provide:

- Goal.
- Scope.
- Ordered issue list.
- Dependencies.
- Expected branch name.
- Completion criteria.
- Validation expectations.

## Branch model

Use a milestone or epic branch:

- `milestone/<short-name>`
- `epic/<short-name>`

All related issue work for that milestone or epic may be built on the branch in the order defined by the milestone.

## Coding rules

- Code changes are allowed only within an approved milestone or epic.
- Follow the ordered issue list.
- Keep unrelated work out.
- Update docs when behaviour or developer usage changes.
- Add tests when practical.
- Do not claim tests passed unless they were actually run.
- Open a milestone PR when the work is ready for review.

## Deferred testing

Local testing may be deferred until the user has Codex running on a laptop.

Every PR with deferred testing must list:

- Tests not run.
- Reason tests were not run.
- Commands to run later.
- Known risk areas.

## Merge rule

A milestone PR may be merged when the user explicitly asks. If local tests have not run, the merge summary must state that validation is deferred.

## Still not enabled

- Background scheduled coding.
- Unreviewed direct commits to `main`.
- Secret, billing, deployment, or production access changes.
