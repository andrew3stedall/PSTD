# Phone-First Operating Model

## Purpose

PSTD planning can be run from ChatGPT on mobile through the GitHub connector.

## Current approach

```text
ChatGPT mobile prompt
  -> GitHub connector
  -> repo context and planning rules
  -> docs, issues, branches, and PRs
  -> mobile review
```

## Why this approach

This keeps the project moving without requiring a local Codex install or an API-backed GitHub Action.

## What works well

- Planning reports.
- PRD decomposition.
- Developer-ready issue design.
- Documentation updates.
- Branch and PR creation.
- Lightweight repo maintenance.

## Limits

This mode does not provide scheduled runs, local test execution, or true parallel subagent execution.

## Working loop

1. User gives a requirement, PRD, bug report, or design goal in ChatGPT.
2. Assistant reads relevant repo files through the GitHub connector.
3. Assistant applies `AGENTS.md` and the repo skill instructions.
4. Assistant creates or updates docs, issues, branches, or PRs.
5. User reviews from GitHub mobile or ChatGPT.
6. Assistant refines based on feedback.

## Repo skills

The reusable skill files live under `.agents/skills/`.

## Human checkpoints

Human review is required before moving from planning-only to implementation mode, enabling paid automation, or enabling parallel subagents.
