# ADR-0002: Use a mobile planning workflow

## Status

Accepted

## Context

The user is currently working from ChatGPT on mobile. The repo still needs a practical loop for PRDs, issues, docs, and PRs.

## Decision

Use ChatGPT plus the GitHub connector as the current operating path.

The repo will keep:

- `AGENTS.md` for repo-level instructions.
- `.agents/skills/` for reusable planning skills.
- `docs/` for product, engineering, data, and decision records.
- GitHub issues and PRs as the delivery record.

The repo will not rely on a key-based planning workflow by default.

## Consequences

### Positive

- Works from mobile.
- Keeps planning visible in GitHub.
- Allows the process to mature before execution mode.

### Negative

- No unattended loop.
- No local test execution from the mobile path.
- Requires explicit user prompts to continue work.
- Some GitHub settings still require manual action in the GitHub UI.

## Revisit when

- A local Codex app is available.
- The repo has CI and validation commands.
- Planning-only mode is ready to become single-task execution mode.
