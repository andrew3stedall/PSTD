# PSTD v1 M18 Implementation Plan

## Implementation intent

M18 adds a review workflow layer for decoder backlog records. The implementation should help future parser work start from reviewed evidence and focused regression coverage, rather than broad speculative decoding.

## Current foundation

- M16 exports `data/compatibility_triage.jsonl`.
- M17 exports `data/decoder_backlog.jsonl`.
- M17 backlog rows already include category, priority, severity, observed count, and recommended action.

## Implemented M18 slice

1. Review summary:
   - Adds `DecoderBacklogReviewSummary`.
   - Counts total backlog rows, high/medium/low priority rows, decoder work, payload mapping work, and unique candidates.
   - Emits a review status that makes empty, low, medium, and high priority states explicit.
2. Issue candidates:
   - Adds `DecoderIssueCandidate`.
   - Groups backlog rows by decoder candidate key.
   - Preserves affected message count, observed total, source item count, recommended title, recommended action, and checklist.
3. Extraction output:
   - Writes `data/decoder_backlog_review.jsonl`.
   - Writes `data/decoder_issue_candidates.jsonl`.
   - Adds manifest rows and status counters for the review outputs.
4. Tests:
   - Covers non-empty summaries.
   - Covers empty summaries.
   - Covers issue candidate generation and checklist content.

## Output fields

### `DecoderBacklogReviewSummary`

- `run_id`
- `pst_id`
- `total_items`
- `high_priority_count`
- `medium_priority_count`
- `low_priority_count`
- `decoder_work_count`
- `payload_mapping_count`
- `unique_candidate_count`
- `top_candidate_key`
- `review_status`

### `DecoderIssueCandidate`

- `run_id`
- `pst_id`
- `decoder_candidate_key`
- `category`
- `priority`
- `backlog_status`
- `affected_message_count`
- `observed_total`
- `source_item_count`
- `recommended_title`
- `recommended_action`
- `checklist`
- `issue_status`

## Review rule

A decoder issue should not be implemented until the issue candidate has been reviewed and a focused regression test exists or is planned as part of the same implementation.

## Definition of done

- M18 branch keeps CI green.
- Review outputs are emitted in extraction archives.
- Docs explain how to use the review outputs.
- Project status and changelog are updated.
