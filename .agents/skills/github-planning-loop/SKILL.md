---
name: github-planning-loop
description: Use when operating PSTD from ChatGPT on mobile through the GitHub connector. This skill defines a phone-first loop for planning, issue creation, docs updates, and PR creation without API-billed GitHub Actions.
---

# GitHub Planning Loop Skill

## Purpose

Use this skill when work is being coordinated from ChatGPT on a phone rather than from a local Codex app or API-backed workflow.

## Loop

1. User provides a PRD, idea, issue, or feedback in ChatGPT.
2. Assistant reads relevant repo files through the GitHub connector.
3. Assistant applies `planning-council` logic in the chat.
4. Assistant creates or updates docs, issues, or PRs through the GitHub connector.
5. User reviews from GitHub mobile or ChatGPT.
6. Assistant refines based on feedback.

## Allowed work

- Planning reports.
- Docs updates.
- Issue creation and refinement.
- Template maintenance.
- PR creation for planning and documentation changes.
- Small repo metadata changes where the connector supports them.

## Not provided by this loop

- Unattended background execution.
- Scheduled automation.
- Local test execution.
- API-key GitHub Actions usage.
- Parallel subagent execution.

## Working rule

Do the useful repo work that can be completed through the GitHub connector. Be explicit about anything that requires local execution, API billing, GitHub settings access, or a future Codex runtime.
