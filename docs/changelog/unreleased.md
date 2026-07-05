# Unreleased

## Added

- PQ3 index entry decoding for PST conversion coverage.
- Page-level diagnostics in inspect JSON for BBT and NBT traversal.
- Regression coverage for B-tree page metadata, internal child references, and leaf entries.
- PQ3 operations documentation, milestone summary, and issue plan.
- PQ2 root candidate selection for post-v1 parser-quality work.
- Root candidate diagnostics in inspect JSON, including `selected_source`, `candidate_count`, and `candidates[]`.
- Later Unicode root BREF offset candidate evaluation.
- Regression coverage for safe later-root selection and no usable candidate pairs.
- PQ2 operations documentation, milestone summary, and issue plan.
- PQ1 root decode diagnostics for safe real-PST parser-quality assessment.
- `header.root_diagnostics` in inspect JSON with file size, root page size, per-root bounds checks, condition names, and recommendation text.
- `root_diagnostic_condition` at the inspect-summary level.
- Regression coverage for root offsets that decode beyond file size.
- PQ1 operations documentation for interpreting root diagnostic conditions.
- PSTD v1 planning package, milestone documentation, engineering guides, operations guides, and repo skills.
- M1 extraction foundation and structured archive contract.
- M2 PST binary foundation.
- M3 folder and metadata extraction foundation.
- M4 recipients, threading, and address resolution foundation.
- M5 message body and attachment output foundation.
- M6 batch orchestration and resume support.
- M7 parser depth diagnostics.
- M8 bounded traversal expansion.
- M9 payload and subnode traversal foundation.
- M10 payload wiring helpers.
- M11 extraction path integration.
- M12 attachment table and subnode integration.
- M13 parser compatibility coverage.
- M14 recursive subnode layout exploration.
- M15 observed layout triage reporting.
- M16 fixture-backed decoder expansion.
- M17 decoder backlog JSONL reporting.
- M18 decoder backlog review workflow outputs.
- M19 focused candidate selection outputs.
- M20 focused attachment table decoder implementation.
- M21 focused decoder evidence expansion.
- M22 body and header fidelity expansion.
- M23 attachment payload fidelity expansion.
- M24 batch scale, performance, and corruption hardening.
- M25 v1 release-candidate and operator handoff.

## Changed

- Updated BBT and NBT page parsing to read B-tree metadata from byte offsets 488 through 491.
- Updated internal-page traversal to use child BREF offsets.
- Updated leaf-page decoding to read entries from the page body starting at byte 0.
- Updated inspect output to show BBT and NBT page diagnostic counts and JSON arrays.
- Updated project status to park downstream loading until PST conversion coverage is reliable.
- Updated root traversal setup to use selected safe root candidates only.
- Updated inspect text output to show selected root source and candidate count.
- Updated project status to identify PQ2 as the current post-v1 parser-quality lane and PQ3 as the next step.
- Updated project status to reflect M1-M25 implementation status and PQ1 as the current post-v1 parser-quality blocker.
- Updated inspect output to expose root diagnostics before tree traversal succeeds.
- Replaced the stale early roadmap with a bounded M1-M25 v1 milestone lane.
- Expanded CLI CI coverage to include `pstd batch --help`.
- Updated summaries to count extracted attachment payloads rather than only attachment metadata rows.
- Updated attachment status handling to distinguish missing subnode references, unavailable subnode blocks, table parse cases, tables without payloads, and extracted payloads.
- Updated compatibility triage evidence classification so `CATB` and `CATW` compact attachment-table statuses both count as fixture-backed decoder evidence.
- Updated message output contract to preserve transport headers when available.
- Updated HTML body extraction to prefer binary HTML when both binary and Unicode HTML properties are present.
- Updated attachment output contract to preserve metadata for known rows even when payload bytes are unavailable.
- Updated compact attachment table missing-payload handling to emit unavailable records rather than counters only.
- Updated batch CLI output to print discovered, attempted, completed, partial, failed, skipped, and not-run counters.
- Updated batch fail-fast handling to preserve discovered totals and report not-run PST files.

## Removed

- Removed the key-based Codex planning workflow from `.github/workflows/`.

## Notes

- M1-M25, PQ1, and PQ2 have passed GitHub Actions validation.
- The active roadmap is now conversion coverage only; downstream loading is parked.
- PQ3 targets index page entry decoding, not folder/message/body/attachment completion.
- PQ2 selects only root candidates whose BBT and NBT pages are fully in bounds.
- PQ1 does not claim full extraction-quality resolution; it makes root traversal blockers explicit and measurable.
