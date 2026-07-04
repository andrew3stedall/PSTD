# PSTD v1 M21 Implementation Plan

## Implementation intent

M21 makes the M20 focused decoder implementation observable in the compatibility evidence pipeline. M20 added a UTF-16 compact attachment-table decoder with statuses in the `utf16_compact_attachment_table_*` family. M21 ensures those statuses are counted as fixture-backed decoder evidence, not left invisible to the triage workflow.

## Current foundation

- M17 exports row-level decoder backlog records.
- M18 exports backlog review and grouped issue candidates.
- M19 exports candidate selection records.
- M20 adds `CATW` UTF-16 compact attachment-table decoding and focused fallback tests.

## Implemented M21 slice

1. Compatibility module routing:
   - Adds an M21 wrapper around the existing compatibility implementation.
   - Re-exports the existing compatibility structs and functions.
   - Overrides only `triage_observed_attachment_layouts` for the M21 evidence classification.
2. Evidence classification:
   - Counts `compact_attachment_table_*` statuses as existing `CATB` compact evidence.
   - Counts `utf16_compact_attachment_table_*` statuses as M20 `CATW` compact evidence.
   - Emits a distinct `utf16_compact_attachment_table_layout` supported case.
   - Sums both status families into `fixture_backed_decoder_count`.
3. Regression coverage:
   - Adds a test for `CATW` fixture-backed evidence classification.
   - Adds a test for combined `CATB` and `CATW` supported evidence.
4. Handoff:
   - Leaves parser expansion closed for M21.
   - Moves the next implementation milestone to M22 body/header fidelity unless CI exposes a blocking parser issue.

## Compatibility rules

| Status family | Case category | Counted as fixture-backed evidence |
|---|---|---:|
| `compact_attachment_table_*` | `compact_attachment_table_layout` | Yes |
| `utf16_compact_attachment_table_*` | `utf16_compact_attachment_table_layout` | Yes |
| Parse errors | `unparseable_attachment_table` | No |
| Missing payloads | `attachment_rows_without_payloads` | No |
| Unsupported subnode layouts | `unsupported_subnode_layout` | No |

## Definition of done

- M21 branch keeps CI green.
- Compatibility tests prove `CATW` evidence is counted.
- Existing `CATB`, unsupported-layout, parse-error, and missing-payload behaviour remains intact.
- Docs explain the selected evidence gap and the M22 handoff.
