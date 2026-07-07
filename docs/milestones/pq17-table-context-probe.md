# PQ17 Table Context Probe

Scope remains focused on the table-like message subnode source from PQ16.

## Implemented signal

PQ17 surfaces these run-status counters:

- `pq17_table_parse_attempts`
- `pq17_table_parse_successes`
- `pq17_table_parse_failures`
- `pq17_table_columns`
- `pq17_table_rows`
- `pq17_next_blocker`

## Expected public fixture result

The public fixture should remain at 50 BBT entries, 63 NBT entries, 11 folders, 1 message, and 0 attachments. The expected PQ17 signal is one table parse attempt and one parse success from the table-like message subnode source.

## Next expected blocker

If table parsing succeeds but row count remains zero, the next blocker is `table_row_matrix_or_row_count_decode`.
