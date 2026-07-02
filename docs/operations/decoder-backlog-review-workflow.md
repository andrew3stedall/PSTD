# Decoder Backlog Review Workflow

## Purpose

Use this workflow after a run has produced decoder backlog outputs. The goal is to decide which parser work is worth implementing next while preserving explicit fallback behaviour for layouts that are still unsupported.

## Inputs

| File | Use |
|---|---|
| `data/compatibility_triage.jsonl` | Source triage records from observed compatibility cases. |
| `data/decoder_backlog.jsonl` | Row-level backlog records derived from non-supported triage cases. |
| `data/decoder_backlog_review.jsonl` | Run-level summary for priority mix and review status. |
| `data/decoder_issue_candidates.jsonl` | Grouped issue candidates with suggested title, action, and checklist. |

## Review steps

1. Open `data/decoder_backlog_review.jsonl`.
2. Check `review_status`.
3. Start with high-priority candidates from `data/decoder_issue_candidates.jsonl`.
4. For each candidate, review the source `decoder_backlog.jsonl` rows.
5. Confirm the category, observed count, affected item count, and recommended action.
6. Decide whether the candidate has enough evidence for a focused test.
7. Create a GitHub issue only after the candidate has a testable scope.
8. Keep fallback statuses for rows that still cannot be decoded.

## Priority handling

| Priority | Meaning | Action |
|---|---|---|
| High | Parser compatibility work is likely blocking extraction. | Review first and create a focused issue when testable. |
| Medium | Payload mapping or partial extraction path needs review. | Review after high-priority parser blockers. |
| Low | Lower-impact or less-specific parser work. | Hold until stronger evidence exists. |

## Issue creation checklist

Use the checklist from `decoder_issue_candidates.jsonl` as the starting point. A valid implementation issue should include:

- Candidate key.
- Category.
- Priority.
- Source backlog rows or a filtered extract from them.
- Affected item count.
- Observed count.
- Expected regression test.
- Required fallback behaviour.
- Out-of-scope parser changes.

## Do not do

- Do not add broad parser rewrites from a single candidate.
- Do not mark a layout as supported without a test.
- Do not remove fallback statuses for unsupported layouts.
- Do not add generated run artefacts to the repository.
- Do not add Snowflake, search, or web UI work as part of decoder review.

## Handoff to implementation

A candidate is ready for implementation when:

- The category and priority have been reviewed.
- The expected test input shape is clear.
- The parser change can be made in a small, isolated slice.
- The fallback behaviour for non-matching rows remains explicit.
