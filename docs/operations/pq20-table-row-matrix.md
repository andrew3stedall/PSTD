# PQ20 Table Row Matrix Measurement

## Goal

Measure the table row matrix blocker from PQ19.

## Implemented signal

PQ20 adds public progress artifact counters for:

- `pq20_row_matrix_decode_attempts`
- `pq20_parsed_table_columns`
- `pq20_parsed_table_rows`
- `pq20_row_value_slots`
- `pq20_next_blocker`

## Current public-fixture expectation

PQ19 found a table-led path but no membership rows. PQ20 checks whether row and column counters are present in the extraction status after table parsing.

## Boundary

PQ20 does not broaden raw NBT heuristics and does not change extraction output counts. It is an evidence gate for the next implementation step.
