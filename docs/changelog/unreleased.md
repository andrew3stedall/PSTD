# Unreleased

## Added

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
- M21-M25 remaining v1 roadmap tracking issues.
- `data/decoder_backlog_review.jsonl` output.
- `data/decoder_issue_candidates.jsonl` output.
- `data/decoder_candidate_selection.jsonl` output.
- Review checklist guidance for issue candidates.
- Selection scope, test, and fallback guidance for focused candidates.
- Extraction status counters for candidate selections and selected candidates.
- UTF-16 compact attachment table decoding for `CATW` rows.
- Regression coverage for malformed `CATW` rows retaining explicit error status.
- Regression coverage for `CATW` fixture-backed decoder evidence classification.

## Changed

- Updated project status to reflect M1-M21 implementation status.
- Replaced the stale early roadmap with a bounded post-M20 M21-M25 roadmap.
- Updated README, PRD, documentation index, project status, and roadmap to agree that four v1 milestones remain after M21.
- Updated documentation navigation for M21 milestone, implementation, and issue plan.
- Expanded CLI CI coverage to include `pstd batch --help`.
- Updated summaries to count extracted attachment payloads rather than only attachment metadata rows.
- Updated attachment status handling to distinguish missing subnode references, unavailable subnode blocks, table parse cases, tables without payloads, and extracted payloads.
- Updated compatibility triage evidence classification so `CATB` and `CATW` compact attachment-table statuses both count as fixture-backed decoder evidence.

## Removed

- Removed the key-based Codex planning workflow from `.github/workflows/`.

## Notes

- M1-M20 have passed GitHub Actions validation.
- M21 validation is pending on the milestone PR.
- M20 implements one focused selected parser candidate.
- M21 keeps parser expansion narrow and only expands evidence classification for the M20 `CATW` decoder.
- The remaining v1 implementation lane is M22-M25.
- Parser quality still depends on broader observed layout coverage and reviewed validation inputs.
