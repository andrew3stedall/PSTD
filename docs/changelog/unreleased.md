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
- PSTD M2 planning package: M2 milestone, E2 epic, ordered issue plan, dependency map, PST binary implementation plan, deferred testing plan, and GitHub issues #19-#28.
- PSTD M2 implementation: bounded PST byte reader, PST header parser, typed PST primitives, binary parsing helpers, page/block trailer parsers, BBT/NBT skeletons, block loader, real `pstd inspect` wiring, and synthetic byte fixture tests.
- PSTD M3 planning package: M3 milestone, E3 epic, ordered issue plan, dependency map, folder metadata implementation plan, deferred testing plan, and GitHub issues #32-#42.

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
- Updated roadmap and docs index for M2.
- Marked M2 roadmap and milestone docs as implemented and merged to `main`, with validation still deferred.
- Updated roadmap and docs index for M3 as the next planned milestone.

## Removed

- Removed the key-based Codex planning workflow from `.github/workflows/`.

## Notes

- Local testing remains deferred until the laptop/Codex setup is available.
- M1 was merged to `main` without local validation at the user's request.
- M2 was merged to `main` without local validation at the user's request.
- M3 is planning-only; implementation has not started.
- Run the documented Cargo, CLI, Python wrapper, and Docker commands before treating M1 or later milestones as release-verified.
