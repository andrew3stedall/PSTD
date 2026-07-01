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
- M4 recipient and threading milestone documentation.
- M4 selected MAPI properties for message IDs, conversation fields, transport headers, and recipient/address fields.
- M4 threading helpers for subject normalization, reference splitting, and threading status.
- M4 recipient table row conversion into `RecipientRecord` rows.
- Archive output scaffolding for `data/recipients.jsonl` and `data/message_references.jsonl`.
- M5 bodies and attachments milestone documentation.
- M5 selected MAPI properties for text, HTML, RTF, attachment metadata, and attachment data fields.
- M5 body helpers for stable body records, archive paths, byte counts, and SHA-256 hashes.
- M5 attachment helpers for safe filenames, stable attachment records, archive paths, byte counts, and SHA-256 hashes.
- Archive output scaffolding for `data/bodies.jsonl` and `data/attachments.jsonl`.

## Changed

- Refreshed the root README to reflect M1-M3 current status.
- Reorganised `docs/README.md` around audience-based navigation.
- Updated M1, M2, and M3 documentation status to implemented with CI validation.
- Updated output-contract guidance around structured TAR + JSONL output.
- Updated project status to reflect M4 recipients/threading foundation work and CI validation.
- Updated project status to reflect the M5 body and attachment foundation slice.

## Removed

- Removed the key-based Codex planning workflow from `.github/workflows/`.

## Notes

- M1-M4 have passed GitHub Actions validation.
- M5 is designed to emit deterministic body and attachment output records while current parser depth remains limited.
- Current recipient extraction converts parsed table rows into records, but broader real-world PST recipient extraction still depends on deeper BBT/NBT and table traversal coverage.
- Private PST files must not be committed as fixtures.
