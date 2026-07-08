# PQ24 Column Mapping

PQ24 implements issue #344.

## Expected decision

- If selected values appear, PQ25 should materialize a safe selected property from the table.
- If only plausible values appear, PQ25 should expand the selected dictionary or table column interpretation.
- If only unknown values remain, PQ25 should diagnose the table column tag shape.
