# PQ22 Aggregate Table Counts

## Goal

Surface aggregate table row, column, and value counters in run-level extraction status.

## Scope

- Sum table counters from message status fields.
- Append aggregate `pq21_table_*` counters to run summary status.
- Keep extraction outputs unchanged until row mapping is validated.

## Boundary

PQ22 does not map table rows to properties. It only makes the measured row and value counts visible at run level.
