# AGENTS.md

## Repository purpose

PSTD is a PST email data extractor. The current stated direction is to use Rust to process PST files and extract individual emails. Future planning may include Python, React/Vite, and Snowflake components, but planning must not assume those components exist until the repository contains them.

## Operating model

This repository uses a planning-first Codex Delivery Council model. Codex should act as a bounded planning agent, not an unrestricted autonomous developer.

The current operating path is phone-first:

- The user prompts from ChatGPT on mobile.
- The assistant uses the GitHub connector to read and update repo artefacts.
- Planning, docs, issues, and PRs are created through GitHub, not through an API-key GitHub Action.
- Repo-scoped skills live under `.agents/skills/` for future Codex runtimes and as reusable planning instructions.

## Current mode

Mode: `planning-only`.

For now the assistant/Codex is allowed to:

- Read draft product requirements and repo context.
- Convert draft PRDs into epics, milestones, and GitHub issues.
- Define acceptance criteria, dependencies, risks, and documentation requirements.
- Refine issues after developer feedback.
- Produce planning reports and documentation.
- Update planning and documentation files.
- Open PRs for planning and documentation changes.

For now the assistant/Codex is not allowed to:

- Implement application code unless explicitly moved out of planning-only mode.
- Auto-merge changes.
- Run parallel subagents.
- Rely on API-key GitHub Actions for Codex work.
- Modify deployment, credentials, authentication, billing, or production data flows.
- Create broad architecture without an ADR.

## Skills

Use `.agents/skills/README.md` as the skills index.

Core skills:

- `.agents/skills/planning-council/SKILL.md` for PRD-to-plan work.
- `.agents/skills/issue-writer/SKILL.md` for developer-ready issue design.
- `.agents/skills/docs-writer/SKILL.md` for documentation maintenance.
- `.agents/skills/github-planning-loop/SKILL.md` for ChatGPT mobile plus GitHub connector workflows.

Role skills:

- `roles/executive-sponsor`
- `roles/product`
- `roles/business-analyst`
- `roles/ux`
- `roles/developer-feasibility`
- `roles/metrics`
- `roles/data`
- `roles/platform`
- `roles/integration`
- `roles/reviewer`

Process skills:

- `process/prd-intake`
- `process/milestone-planner`
- `process/epic`
- `process/dependency-mapper`
- `process/risk-reviewer`
- `process/readiness-check`
- `process/feedback-refiner`

When operating from this ChatGPT conversation, treat those skills as committed instruction files and follow their intent manually.

## Required role sequence for planning work

1. Executive Sponsor: confirm alignment with the original goal.
2. Product: confirm product value and MVP relevance.
3. Business Analyst: decompose requirements into epics, issues, dependencies, and success criteria.
4. UX: define relevant user flows and usability constraints.
5. Metrics: define analysis, anomaly, inference, and evaluation needs where relevant.
6. Data: define data contracts, volume assumptions, batch or streaming needs, and data quality checks.
7. Platform: define operational, CI/CD, environment, and delivery risks.
8. Developer Feasibility: provide implementation feasibility notes only while in planning-only mode.
9. Docs Writer: update the planning, product, architecture, engineering, data, and user documentation structure.
10. Integration: check dependency order and overlap.
11. Reviewer: check readiness and quality.

## Planning rules

- Work from repository evidence and explicit user instructions.
- Do not invent test, build, lint, or typecheck commands when they are unknown.
- If the repo lacks code or tooling, document the gap instead of assuming a stack is already implemented.
- Every planned issue must include scope, out-of-scope items, acceptance criteria, dependencies, risks, and documentation requirements.
- Prefer small, dependency-aware issues over large ambiguous ones.
- Mark work as on hold if product intent, data access, operating model, or acceptance criteria are missing.
- Make requirements developer-ready; a developer should not need extra context to begin.

## Risk controls

Planning may describe risky work, but the assistant/Codex must not execute risky work without explicit human approval. Risky work includes:

- Data deletion or irreversible transformation.
- Processing private or sensitive email content without a privacy and retention design.
- Credential handling.
- Authentication or authorization changes.
- Infrastructure, deployment, or CI/CD permission changes.
- Database migrations.
- Large-scale data extraction or inference pipelines.

## Pull request standard

Every PR must include:

- Purpose.
- Scope.
- Files changed.
- Tests or validation performed.
- Documentation updated.
- Data impact.
- Operational impact.
- Follow-up work.

## Documentation standard

Every meaningful planning change must update docs. Use audience-specific folders under `docs/` rather than placing all notes in one file.
