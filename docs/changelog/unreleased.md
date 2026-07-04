# Unreleased

## Added

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
- M21-M25 remaining v1 roadmap tracking issues.
- `data/decoder_backlog_review.jsonl` output.
- `data/decoder_issue_candidates.jsonl` output.
- `data/decoder_candidate_selection.jsonl` output.
- `batch_progress.jsonl` output for root-level batch operator progress events.
- Batch summary counters for discovered, attempted, completed, partial, failed, skipped, and not-run PST files.
- v1 release-candidate checklist.
- Local and Docker operator handoff runbook.
- Unsupported and deferred area documentation.
- Review checklist guidance for issue candidates.
- Selection scope, test, and fallback guidance for focused candidates.
- Extraction status counters for candidate selections and selected candidates.
- UTF-16 compact attachment table decoding for `CATW` rows.
- Unicode/string HTML body extraction for reachable `PR_HTML` string properties.
- `transport_message_headers` on message records when `PR_TRANSPORT_MESSAGE_HEADERS` is available.
- Attachment record fields for declared size, size status, and attachment method.
- Metadata-only attachment records for unavailable, empty, or deferred payload rows.
- Regression coverage for malformed `CATW` rows retaining explicit error status.
- Regression coverage for `CATW` fixture-backed decoder evidence classification.
- Regression coverage for Unicode HTML body extraction and transport-header metadata.
- Regression coverage for attachment metadata-only rows and declared-size status handling.
- Regression coverage for batch counter aggregation and partial-success status classification.

## Changed

- Updated project status to reflect M1-M25 implementation status and PQ1 as the current post-v1 parser-quality blocker.
- Updated inspect output to expose root diagnostics before tree traversal succeeds.
- Replaced the stale early roadmap with a bounded M1-M25 v1 milestone lane and post-v1 Snowflake ingestion planning boundary.
- Updated README, PRD, documentation index, project status, and roadmap to agree that the planned v1 milestone lane is complete after M25.
- Updated documentation navigation for M25 milestone, implementation, issue plan, release-candidate checklist, operator handoff, and unsupported/deferred areas.
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
- Updated output contract notes to describe the v1 release-candidate boundary and post-v1 planning handoff.

## Removed

- Removed the key-based Codex planning workflow from `.github/workflows/`.

## Notes

- M1-M25 have passed GitHub Actions validation.
- PQ1 does not claim full extraction-quality resolution; it makes root traversal blockers explicit and measurable.
- M20 implements one focused selected parser candidate.
- M21 keeps parser expansion narrow and only expands evidence classification for the M20 `CATW` decoder.
- M22 keeps body/header fidelity narrow and only expands already-reachable body/header properties.
- M23 keeps attachment fidelity narrow and preserves already-reachable attachment metadata for unavailable/deferred payloads.
- M24 keeps batch hardening local/Docker focused and does not add distributed orchestration.
- M25 closes the planned v1 milestone lane once CI passes and the PR is merged.
- Parser quality still depends on broader observed layout coverage and reviewed validation inputs.
