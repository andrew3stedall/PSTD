# PQ18 Table Property Candidates

## Goal

Measure whether table rows can become alternate message property candidates after PQ17.

## Implemented signal

PQ18 surfaces:

- `pq18_candidate_rows`
- `pq18_candidate_values`
- `pq18_selected_property_lift`
- `pq18_plausible_property_lift`
- `pq18_next_blocker`

## Current public-fixture expectation

PQ17 showed one table parse success but zero table rows. PQ18 should therefore report zero candidate rows and keep the blocker on row-matrix or row-count decoding.

## Boundary

PQ18 does not broaden raw NBT heuristics and does not claim extraction lift until decoded table rows exist.
