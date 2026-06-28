# AGENTS.md

## Repository purpose

PSTD is a PST email data extractor. The current stated direction is to use Rust to process PST files and extract individual emails. Future planning may include Python, React/Vite, and Snowflake components, but planning must not assume those components exist until the repository contains them.

## Operating model

This repository uses a Codex Delivery Council model with two lanes:

1. Planning lane: turn product intent into milestones, epics, issues, docs, and risks.
2. Execution lane: build approved milestones or epics on milestone branches.

The current operating path is phone-first:

- The user prompts from ChatGPT on mobile.
- The assistant uses the GitHub connector to read and update repo artefacts.
- Planning, docs, issues, branches, and PRs are created through GitHub.
- Local testing may be deferred until the user has Codex running on a laptop.
- Repo-scoped skills live under `.agents/skills/` for future Codex runtimes and as reusable instructions.

## Current mode

Mode: `milestone-execution`.

Allowed:

- Read product requirements and repo context.
- Convert PRDs into epics, milestones, and GitHub issues.
- Define acceptance criteria, dependencies, risks, and documentation requirements.
- Build an approved milestone or epic without requiring a new prompt for every issue.
- Follow the ordered issue list defined by the milestone or epic.
- Use milestone or epic branches.
- Update application code within the approved milestone or epic scope.
- Add or update tests where practical.
- Document tests that were not run locally.
- Update docs and open PRs.
- Merge PRs when the user explicitly asks.

Not allowed:

- Unrelated broad refactors.
- Direct commits to `main` unless explicitly requested.
- Secret, billing, deployment, authentication, or production access changes.
- Claiming tests passed when they were not run.
- Creating broad architecture without an ADR.

## Skills

Use `.agents/skills/README.md` as the skills index.

Core skills:

- `.agents/skills/planning-council/SKILL.md`
- `.agents/skills/issue-writer/SKILL.md`
- `.agents/skills/docs-writer/SKILL.md`
- `.agents/skills/github-planning-loop/SKILL.md`

Role skills:

- `roles/executive-sponsor`
- `roles/product`
- `roles/business-analyst`
- `roles/ux`
- `roles/developer-feasibility`
- `roles/full-stack-developer`
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

Execution skills:

- `execution/milestone-executor`
- `execution/epic-workforce`
- `execution/implementation-worker`
- `execution/milestone-branch-manager`
- `execution/deferred-testing`

## Planning role sequence

1. Executive Sponsor.
2. Product.
3. Business Analyst.
4. UX.
5. Metrics.
6. Data.
7. Platform.
8. Developer Feasibility.
9. Full-Stack Developer.
10. Docs Writer.
11. Integration.
12. Reviewer.

## Execution workflow

1. Start from an approved milestone or epic.
2. Confirm the ordered issue list.
3. Create or use a milestone branch.
4. Implement the issue set in the milestone order.
5. Keep unrelated work out.
6. Add or update tests when practical.
7. Record tests that could not be run.
8. Update docs.
9. Open a milestone PR.
10. Merge when the user explicitly asks.

## Planning rules

- Work from repository evidence and explicit user instructions.
- Do not invent test, build, lint, or typecheck commands when they are unknown.
- If the repo lacks code or tooling, document the gap instead of assuming a stack is already implemented.
- Every planned issue must include scope, out-of-scope items, acceptance criteria, dependencies, risks, and documentation requirements.
- Prefer small, dependency-aware issues within coherent milestones.
- Mark work as on hold if product intent, data access, operating model, or acceptance criteria are missing.

## Pull request standard

Every PR must include:

- Purpose.
- Scope.
- Files changed.
- Tests or validation performed.
- Tests or validation deferred.
- Documentation updated.
- Data impact.
- Operational impact.
- Follow-up work.

## Documentation standard

Every meaningful planning or execution change must update docs. Use audience-specific folders under `docs/` rather than placing all notes in one file.
