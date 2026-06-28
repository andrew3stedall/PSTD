# ADR-0003: Use milestone execution mode

## Status

Accepted

## Context

The initial PSTD setup focused on planning. The user wants to build from a phone-first workflow without prompting for every issue. Local tests will be handled later when the user has Codex running on a laptop.

## Decision

Use milestone execution mode.

The execution unit is an approved milestone or epic. The milestone or epic defines the ordered issue set, branch, scope, and completion criteria.

Implementation may proceed across the ordered issues in that milestone without requiring a new prompt for each issue.

## Rules

- Build from an approved milestone or epic.
- Use a milestone or epic branch.
- Follow the ordered issue list.
- Keep unrelated work out of the branch.
- Record tests that were not run.
- Open a milestone PR for review and merge only when requested.

## Consequences

### Positive

- Reduces prompting overhead.
- Allows coherent milestone delivery.
- Keeps work traceable to a defined scope.
- Supports phone-first development until local Codex is available.

### Negative

- Larger PRs may be harder to review.
- Local validation is deferred.
- Milestone definitions must be high quality before coding starts.

## Revisit when

- Codex laptop setup is available.
- CI exists.
- Milestone PRs become too large to review confidently.
