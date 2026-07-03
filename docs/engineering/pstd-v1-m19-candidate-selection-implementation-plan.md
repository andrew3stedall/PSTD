# PSTD v1 M19 Implementation Plan

## Implementation intent

M19 makes the M18 review workflow actionable by producing a deterministic candidate selection output. The output is designed to guide the next implementation milestone while keeping decoder expansion narrow and test-first.

## Current foundation

- M17 exports row-level backlog records.
- M18 exports review summaries and grouped issue candidates.
- M18 issue candidates include priority, category, affected count, observed total, recommendation, and checklist.

## Implemented M19 slice

1. Selection model:
   - Adds `DecoderCandidateSelection`.
   - Produces one selection record per M18 issue candidate.
   - Preserves rank, candidate key, category, priority, affected count, observed total, and source item count.
2. Selection rules:
   - High priority before medium before low.
   - Higher observed total first.
   - Higher affected message count next.
   - Candidate key as final deterministic tie-breaker.
3. Selection status:
   - High priority candidates are `selected_for_next_planning`.
   - Medium priority candidates are `hold_for_high_priority_review`.
   - Low priority candidates are `hold_for_more_evidence`.
4. Candidate guidance:
   - Adds implementation scope text.
   - Adds test expectation text.
   - Adds fallback requirement text.
5. Extraction output:
   - Writes `data/decoder_candidate_selection.jsonl`.
   - Adds manifest row for candidate selection output.
   - Adds extraction status counters for candidate selections and selected candidates.

## Output fields

- `run_id`
- `pst_id`
- `selection_rank`
- `decoder_candidate_key`
- `category`
- `priority`
- `affected_message_count`
- `observed_total`
- `source_item_count`
- `selection_status`
- `selection_reason`
- `recommended_next_step`
- `implementation_scope`
- `test_expectation`
- `fallback_requirement`

## Definition of done

- M19 branch keeps CI green.
- Candidate selection tests pass.
- Extraction archive includes `data/decoder_candidate_selection.jsonl`.
- Docs explain how to use selection output to seed the next implementation milestone.
