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
- Skill references for output contract, error policy, Rust structure, CLI design, and module boundaries.
- Skill assets for manifest, message metadata, error logs, extraction summaries, milestone PRs, and deferred testing.
- PSTD v1 planning package, roadmap, M1 milestone, E1 epic, ordered issue plan, dependency map, output contract summary, implementation plan, deferred testing plan, and execution checklist.
- PSTD M1 foundation implementation: `pstd` Rust crate, CLI shell, placeholder engine, TAR shard writer, JSONL writer, stable ID helpers, safe path helpers, output record models, status/progress records, PST placeholder modules, Python wrapper boundary, Docker scaffold, fixture guidance, and smoke/unit test placeholders.

## Changed

- Updated `AGENTS.md` for ChatGPT mobile plus GitHub connector operation.
- Updated `AGENTS.md` to reference core, role, process, and execution skills.
- Updated `AGENTS.md` to allow approved milestone or epic execution.
- Updated `.agents/skills/README.md` to link skill references and assets.
- Updated README to point to the `pstd` M1 scaffold and validation commands.
- Updated planning labels to use connector-friendly alternatives where required.
- Updated operating policy to avoid key-based planning workflows by default.
- Updated output-contract skill reference to use structured TAR + JSONL as the v1 canonical output instead of EML-first output.
- Marked M1 roadmap and milestone docs as implemented with validation deferred.

## Removed

- Removed the key-based Codex planning workflow from `.github/workflows/`.

## Notes

- Local testing remains deferred until the laptop/Codex setup is available.
- M1 was merged to `main` without local validation at the user's request.
- Run the documented Cargo, CLI, Python wrapper, and Docker commands before treating M1 as release-verified.
