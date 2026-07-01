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
- M6 batch and resume milestone documentation.
- `pstd batch` command for local multi-PST batch execution.
- Recursive PST discovery and per-PST output directory planning.
- Batch checkpoint output at `batch_checkpoint.jsonl`.
- Batch summary output at `batch_summary.json`.
- Resume-by-skip behaviour for completed per-PST outputs.
- M7 parser depth hardening milestone documentation.
- BBT page diagnostics for parsed entries, truncated entries, page type, page level, and duplicate block entries.
- Node-index page diagnostics for parsed entries, truncated entries, page type, page level, and duplicate node entries.
- Synthetic parser diagnostics tests for complete and truncated BBT/node-index pages.
- M8 traversal expansion milestone documentation.
- Bounded BBT traversal from internal pages to leaf pages.
- Bounded node-index traversal from internal pages to leaf pages.
- Traversal status for parsed pages, discovered child pages, repeated offsets, and traversal errors.
- Table-context parse reports for declared/parsed rows and columns, truncated rows and columns, and omitted values.
- Property-context parse reports for selected, unknown, skipped, and decode-error counts.
- M9 payload and subnode traversal milestone documentation.
- Parser limits for B-tree traversal pages, payload block size, and subnode depth.
- BBT/NBT traversal APIs that accept explicit parser limits.
- BBT-backed payload block loading with a payload size cap.
- Body payload builders from parsed property contexts.
- Attachment payload builders from parsed property contexts.
- Subnode-reference reporting from node-index entries.
- M10 payload wiring milestone documentation.
- Node data-block to property-context wiring helpers.
- Structured node payload reports.
- Bounded subnode decode planning with depth-limit status.
- Attachment table row to property-context conversion.
- Attachment table row to payload construction with wiring reports.

## Changed

- Refreshed the root README to reflect M1-M3 current status.
- Reorganised `docs/README.md` around audience-based navigation.
- Updated M1, M2, and M3 documentation status to implemented with CI validation.
- Updated output-contract guidance around structured TAR + JSONL output.
- Updated project status to reflect M4 recipients/threading foundation work and CI validation.
- Updated project status to reflect the M5 body and attachment foundation slice and CI validation.
- Updated project status to reflect the M6 batch orchestration foundation slice and CI validation.
- Updated project status to reflect the M7 parser-depth hardening slice and CI validation.
- Updated project status to reflect the M8 traversal expansion slice and CI validation.
- Updated project status to reflect the M9 payload and subnode traversal slice and CI validation.
- Updated project status to reflect the M10 payload wiring slice.
- Expanded CLI CI coverage to include `pstd batch --help`.
- Expanded BBT/NBT status strings with traversal diagnostics used by inspect and extract status.

## Removed

- Removed the key-based Codex planning workflow from `.github/workflows/`.

## Notes

- M1-M9 have passed GitHub Actions validation.
- M10 adds payload wiring helpers while deeper subnode decoding remains a later slice.
- Real-world payload extraction still depends on deeper subnode decoding and main extraction-path integration.
- Private PST files, batch checkpoints, and extracted content must not be committed as fixtures unless explicitly sanitized.
