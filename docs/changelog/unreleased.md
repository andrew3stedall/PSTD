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
- M11 extraction path integration milestone documentation.
- Main processing-path node payload loading attempts.
- Body payload metadata and TAR file writing when payload properties are available.
- Explicit unavailable body rows for unsupported payload paths.
- Per-payload manifest rows for body and attachment payloads.
- M12 attachment table and subnode integration milestone documentation.
- Bounded subnode root-block loading with status reports.
- Attachment-table parsing attempts from loaded subnode blocks.
- Attachment payload extraction from parsed subnode table rows.
- Main processing-path attachment extraction through matching subnode references.
- Synthetic tests for subnode loading and subnode attachment table conversion.
- M13 payload fixture expansion and parser compatibility milestone documentation.
- Attachment subnode compatibility diagnostics for parse-error offsets, parse-error reasons, and parsed table statuses.
- Synthetic mixed-block attachment compatibility coverage for extracted, unparseable, and missing-payload table blocks.
- Synthetic body payload fixture coverage for text, HTML, and RTF payload paths.
- M14 recursive subnode layout exploration milestone documentation.
- Subnode layout classification reports for table-compatible, known child-reference, and unsupported layouts.
- Bounded recursive subnode child loading for known child-reference layouts.
- Recursive subnode extraction-path counters for child references, child decodes, unsupported layouts, and table parse errors.
- Synthetic tests for layout classification, mixed layout compatibility, recursive child loading, and depth-limit behaviour.
- M15 observed layout compatibility and public fixture triage milestone documentation.
- Observed layout triage reports for supported, partial, and parser-work cases.
- Compatibility triage categories for table layouts, child-reference layouts, unsupported subnode layouts, unparseable attachment tables, and attachment rows without payloads.
- Public and sanitized fixture triage guide.
- Synthetic tests for supported layouts, unsupported layouts, parse errors, missing payloads, and empty triage reports.
- M16 fixture-backed decoder expansion milestone documentation.
- Compact `CATB` attachment-table decoder with focused regression coverage.
- Compatibility triage records exported as `data/compatibility_triage.jsonl`.
- Compatibility triage classification for compact attachment-table decoder hits.
- Extraction status counters for fixture-backed decoder hits, compatibility triage records, and follow-up cases.
- M17 compatibility triage reporting and decoder backlog milestone documentation.
- Decoder backlog records derived from non-supported compatibility triage cases.
- Decoder backlog priority mapping for parser and payload mapping work.
- Decoder backlog records exported as `data/decoder_backlog.jsonl`.
- Extraction status counter for decoder backlog items.

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
- Updated project status to reflect the M10 payload wiring slice and CI validation.
- Updated project status to reflect the M11 extraction integration slice and CI validation.
- Updated project status to reflect the M12 attachment subnode integration slice and CI validation.
- Updated project status to reflect the M13 fixture compatibility slice and CI validation.
- Updated project status to reflect the M14 recursive subnode layout exploration slice and CI validation.
- Updated project status to reflect the M15 observed layout triage slice and CI validation.
- Updated project status to reflect the M16 fixture-backed decoder expansion slice and CI validation.
- Updated project status to reflect the M17 triage reporting slice.
- Expanded CLI CI coverage to include `pstd batch --help`.
- Expanded BBT/NBT status strings with traversal diagnostics used by inspect and extract status.
- Updated summaries to count extracted attachment payloads rather than only attachment metadata rows.
- Updated attachment status handling to distinguish missing subnode references, unavailable subnode blocks, unparseable tables, tables without payloads, and extracted payloads.

## Removed

- Removed the key-based Codex planning workflow from `.github/workflows/`.

## Notes

- M1-M16 have passed GitHub Actions validation.
- M17 adds decoder backlog JSONL reporting derived from compatibility triage.
- Extraction quality still depends on broader observed PST layout coverage and safe fixture validation.
- Generated extraction outputs and local processing artefacts should not be committed unless intentionally sanitized and reviewed.
