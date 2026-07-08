# PQ23 Table Row Property Candidates

## Goal

Measure safe row-to-property candidate availability after PQ22 exposed decoded table rows and values.

## Implemented signal

PQ23 adds run-status counters for:

- `pq23_candidate_rows`
- `pq23_candidate_values`
- `pq23_selected_property_candidates`
- `pq23_plausible_property_candidates`
- `pq23_next_blocker`

## Boundary

PQ23 does not change message extraction fields. Selected and plausible property candidates remain zero until table column/tag mapping is implemented and validated.
