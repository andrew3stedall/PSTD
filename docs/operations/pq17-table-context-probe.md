# PQ17 Table Context Probe

## Goal

PQ17 should measure table-context parsing for the table-like message subnode source identified by PQ16.

## Acceptance signal

The public PST artifact should expose safe counters for:

- table parse attempts
- table parse successes
- table parse failures
- parsed table columns
- parsed table rows

## Current blocker

`message_subnode_table_payload_wiring`
