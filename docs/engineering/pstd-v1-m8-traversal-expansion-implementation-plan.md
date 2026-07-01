# PSTD v1 M8 Implementation Plan

## Implementation intent

M8 expands traversal depth in a controlled way. The implementation remains bounded and diagnostic-first so later parser work can rely on clearer traversal status.

## Current foundation

- M7 added BBT and node-index page diagnostics.
- M7 status strings expose entry counts, page level, page type, duplicate counts, and truncation counts.
- M1-M7 CI is green.

## Implemented M8 slice

1. BBT bounded traversal:
   - Root pages with a non-zero page level discover child-page candidates.
   - Child page offsets are queued breadth-first.
   - Repeated offsets are skipped.
   - Traversal is capped at 128 pages.
   - Leaf-page entries are appended to the index.
   - Child-page parse failures increment traversal-error counts.
2. Node-index bounded traversal:
   - Same breadth-first traversal model as BBT.
   - Internal node-index entries expose child-page candidates.
   - Leaf node entries are appended to the index.
3. Table-context parse reports:
   - Declared and parsed column counts.
   - Declared and parsed row counts.
   - Truncated column and row counts.
   - Omitted value counts.
4. Property-context parse reports:
   - BTH entry count.
   - Parsed property count.
   - Selected property count.
   - Unknown property count.
   - Skipped-key count.
   - Decode-error count.
5. Synthetic tests for traversal and parse reports.

## Operational behaviour

- `pstd inspect` and extraction status automatically include richer traversal status through the existing index status fields.
- Existing `parse` APIs are preserved for table and property contexts.
- New `parse_with_report` and `from_bth_with_report` APIs expose diagnostics for later milestones.

## Remaining work

- Validate BBT and node-index traversal against broader synthetic fixture coverage.
- Add stricter page-type and level validation once fixture evidence is broader.
- Connect deeper property-context traversal to real node data blocks.
- Connect table-context traversal to real folder, recipient, and attachment table nodes.
- Add attachment subnode traversal.

## Definition of done

- M8 branch keeps CI green.
- BBT and node-index traversal tests pass.
- Table and property parse-report tests pass.
- Docs describe remaining traversal limitations accurately.
