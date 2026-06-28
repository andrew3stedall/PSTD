# Planning Council Overview

PSTD uses a planning-first council model.

## Current mode

Planning only. The council can decompose ideas, define scope, draft epics, design issues, and maintain docs. It should not implement application code in this phase.

## Current operating path

The current path is phone-first rather than API-action-first:

```text
User prompt in ChatGPT mobile
  -> GitHub connector reads repo context
  -> Planning council logic is applied in chat
  -> Docs, issues, and PRs are updated through GitHub
  -> User reviews from mobile
```

## Roles

- Executive Sponsor: protects project alignment.
- Product Owner: checks value and priority.
- Business Analyst: writes epics, issues, dependencies, and success criteria.
- UX Designer: defines user and developer flows.
- Data Scientist: defines metrics and evaluation needs.
- Data Engineer: defines data contracts and quality checks.
- Systems Engineer: defines operational constraints.
- Developer: gives feasibility feedback.
- Docs Writer: keeps documentation current.

## Flow

1. Receive draft PRD or planning request.
2. Check alignment.
3. Confirm product value.
4. Break work into epics and issues.
5. Add UX, data, systems, test, and docs scope.
6. Mark open questions.
7. Update docs, issues, or PRs through the GitHub connector.
8. Send for human review.

## Repo skills

Repo-scoped skill files live under `.agents/skills/`. They are usable by future Codex runtimes and also serve as reusable instructions for this ChatGPT-based workflow.

## Future expansion

Parallel subagents are deliberately delayed until the repo has labels, milestones, CI, validation commands, and a proven single-task workflow.
