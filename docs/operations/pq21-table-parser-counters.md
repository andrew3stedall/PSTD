# PQ21 Table Parser Counters

## Goal

Carry table parser row, column, and value counts into subnode layout status.

## Implemented counters

- `subnode_table_declared_columns`
- `subnode_table_columns`
- `subnode_table_declared_rows`
- `subnode_table_rows`
- `subnode_table_row_width`
- `subnode_table_values`
- `subnode_table_omitted_values`

## Boundary

PQ21 only exposes counters. It does not map table rows to message properties.
