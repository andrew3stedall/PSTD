# PQ24 Column Mapping

## Goal

Classify decoded table columns against safe MAPI property signals.

## Scope

- Count selected table columns and values.
- Count plausible known-value-type columns and values.
- Count unknown columns and values.
- Surface run status under `pq24_status`.

## Boundary

PQ24 measures mapping only. It does not materialize table values into message output fields.
