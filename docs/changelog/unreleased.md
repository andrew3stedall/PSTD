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
- `data/decoder_backlog_review.jsonl` output.
- `data/decoder_issue_candidates.jsonl` output.
- Review checklist guidance for decoder issue candidates.
- Extraction status counters for decoder issue candidates and review status.

## Changed

- Updated project status to reflect M1-M18 CI validation.
- Updated documentation navigation for M18 milestone, implementation, issue plan, and operations guide.
- Expanded CLI CI coverage to include `pstd batch --help`.
- Updated summaries to count extracted attachment payloads rather than only attachment metadata rows.
- Updated attachment status handling to distinguish missing subnode references, unavailable subnode blocks, table parse cases, tables without payloads, and extracted payloads.

## Removed

- Removed the key-based Codex planning workflow from `.github/workflows/`.

## Notes

- M1-M18 have passed GitHub Actions validation.
- M18 adds review workflow outputs derived from decoder backlog rows.
- Parser quality still depends on broader observed layout coverage and reviewed validation inputs.
