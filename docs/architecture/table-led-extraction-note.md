# Table-Led Extraction Note

PSTD should treat table contexts as the main source for user-visible membership once the parser can decode them.

## Current signal

PQ16 classified one decoded message subnode block as table-like.

## Architecture direction

- Use table-context parsing for the table-like subnode source.
- Measure parse attempts, successes, failures, rows, and columns.
- Use later milestones to connect table rows to property and membership extraction.

## Reference

See `docs/research/pst-parser-research.md`.
