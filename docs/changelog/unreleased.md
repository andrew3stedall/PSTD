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
- PSTD v1 planning package and M1-M3 implementation docs.
- Project status guide.
- Developer guide.
- Codebase map.
- System overview.
- Local validation guide.
- Repo-hosted wiki home and developer onboarding pages.

## Changed

- Refreshed the root README to reflect M1-M3 current status.
- Reorganised `docs/README.md` around audience-based navigation.
- Updated M1, M2, and M3 documentation status to implemented with validation deferred.
- Updated output-contract guidance around structured TAR + JSONL output.

## Removed

- Removed the key-based Codex planning workflow from `.github/workflows/`.

## Notes

- Local testing remains deferred until the laptop/Codex setup is available.
- M1, M2, and M3 were merged to `main` without local validation at the user's request.
- Run the documented validation commands before treating M1 or later milestones as release-verified.
