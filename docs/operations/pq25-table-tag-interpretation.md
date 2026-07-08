# PQ25 Table Tag Interpretation

## Goal

Diagnose the two unknown decoded table column tags from PQ24 before attempting property materialization.

## Scope

- Count byte-swapped selected/plausible tag signals.
- Count low-word and high-word known value-type signals.
- Surface run status under `pq25_status`.

## Boundary

PQ25 remains diagnostic-only. It does not materialize table values into message fields.
