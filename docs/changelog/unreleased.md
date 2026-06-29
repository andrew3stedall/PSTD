# Unreleased

## Added

- Planning-only Codex council instructions.
- Planning prompt for role-based decomposition.
- Issue and epic templates.
- Documentation structure for product, engineering, decision, and changelog updates.
- Repo-scoped skills under `.agents/skills/`.
- Mobile-first planning workflow documentation.
- ADR for the mobile planning workflow.
- Role, process, and execution skills under `.agents/skills/`.
- Skill references and assets for planning, output contracts, Rust structure, CLI design, manifests, summaries, and deferred testing.
- PSTD v1 planning package, roadmap, M1 milestone, E1 epic, ordered issue plan, dependency map, output contract summary, implementation plan, deferred testing plan, and execution checklist.
- PSTD M1 foundation implementation.
- PSTD M2 planning package and implementation.
- PSTD M3 planning package: M3 milestone, E3 epic, ordered issue plan, dependency map, folder metadata implementation plan, deferred testing plan, and GitHub issues #32-#42.
- PSTD M3 implementation: logical node access boundary, heap parser, BTH parser, property context scaffold, table context scaffold, selected MAPI property registry, folder inventory output, metadata status rows, metadata-only archive output, and `pstd extract --manifest-only` wiring.

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
- Marked M2 roadmap and milestone docs as implemented and merged to `main`, with validation still deferred.
- Updated roadmap, milestone docs, and docs index for M3 implementation status.

## Removed

- Removed the key-based Codex planning workflow from `.github/workflows/`.

## Notes

- Local testing remains deferred until the laptop/Codex setup is available.
- M1 and M2 were merged to `main` without local validation at the user's request.
- M3 local validation has not yet been run.
- Run the documented Cargo, CLI, Python wrapper, and Docker commands before treating M1 or later milestones as release-verified.
