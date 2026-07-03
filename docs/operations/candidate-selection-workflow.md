# Candidate Selection Workflow

## Purpose

Use this workflow after issue candidates have been produced. The goal is to choose the next focused parser slice using deterministic selection records.

## Inputs

| File | Use |
|---|---|
| `data/decoder_backlog_review.jsonl` | Check the overall priority mix. |
| `data/decoder_issue_candidates.jsonl` | Review grouped candidates and checklists. |
| `data/decoder_candidate_selection.jsonl` | Select the next candidate for focused planning. |

## Selection output

Each selection record includes rank, candidate key, category, priority, affected count, observed total, selection status, selection reason, recommended next step, scope, test expectation, and fallback requirement.

## Selection status guide

| Status | Meaning | Action |
|---|---|---|
| `selected_for_next_planning` | Candidate is high priority and suitable for the next planning pass. | Prepare a focused implementation issue. |
| `hold_for_high_priority_review` | Candidate is useful but should wait while high-priority items exist. | Review after high-priority candidates. |
| `hold_for_more_evidence` | Candidate is low priority or not yet specific enough. | Wait for stronger evidence. |

## Review steps

1. Open `data/decoder_candidate_selection.jsonl`.
2. Pick the lowest `selection_rank` with `selected_for_next_planning`.
3. Read the corresponding record in `data/decoder_issue_candidates.jsonl`.
4. Confirm that the scope is narrow.
5. Confirm that a regression test can be added.
6. Carry the fallback requirement into the implementation issue.
7. Defer lower-ranked candidates until the selected candidate is complete or rejected.

## Handoff template

```text
Candidate key:
Category:
Priority:
Observed total:
Affected count:
Scope:
Test expectation:
Fallback requirement:
Out of scope:
Validation commands:
```

## Do not do

- Do not implement multiple unrelated candidates in one parser milestone.
- Do not remove fallback status while adding a new parser path.
- Do not add search, Snowflake, or web UI work as part of candidate selection.
- Do not treat a held candidate as selected without reviewing the selection reason.
