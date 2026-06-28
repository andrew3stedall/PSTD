# Unreleased

## Added

- Planning-only Codex council instructions.
- Planning prompt for role-based decomposition.
- Issue and epic templates.
- Documentation structure for product, engineering, decision, and changelog updates.
- Repo-scoped skills under `.agents/skills/`.
- Mobile-first planning workflow documentation.
- ADR for the mobile planning workflow.
- Role skills for sponsor, product, analysis, UX, feasibility, full-stack implementation planning, metrics, data, platform, integration, and review.
- Process skills for PRD intake, milestone planning, epic planning, dependency mapping, risk review, readiness checks, and feedback refinement.
- Skills index at `.agents/skills/README.md`.
- Milestone execution skills under `.agents/skills/execution/`.
- Milestone execution policy.
- ADR for milestone execution mode.

## Changed

- Updated `AGENTS.md` for ChatGPT mobile plus GitHub connector operation.
- Updated `AGENTS.md` to reference core, role, process, and execution skills.
- Updated `AGENTS.md` to allow approved milestone or epic execution.
- Updated README to point to repo-scoped skills and the mobile workflow.
- Updated planning labels to use connector-friendly alternatives where required.
- Updated operating policy to avoid key-based planning workflows by default.

## Removed

- Removed the key-based Codex planning workflow from `.github/workflows/`.

## Notes

- Local testing may be deferred until the laptop setup is available.
- Milestone PRs must state tests that were not run and list follow-up validation commands.
