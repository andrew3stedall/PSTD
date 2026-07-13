# PQ69 — Resolve bounded row payload candidates

## Context

PQ68 added a fail-closed bridge from `TcSubnodeRowResolutionReport` to validated row transport evidence. The proposed next step was to invoke that bridge directly from `tc_reporting`.

Review of the production call site showed a missing invariant: `resolve_subnode_row_storage` reports counts and layout evidence, but the matching payload identities are not retained. Reconstructing those payloads inside `tc_reporting` would duplicate the private Unicode SLBLOCK parsing logic and risk count drift between the resolver and bridge.

PQ69 therefore narrows the scope to one reusable payload-candidate resolver. This is a prerequisite for safe call-site integration, not a reporting change.

## Implemented contract

`resolve_row_payload_candidates` accepts:

- the decoded payload collection;
- the TC row NID.

It:

1. scans only structurally valid Unicode leaf SLBLOCK payloads;
2. bounds parsing by both the declared entry count and available complete entries;
3. collects non-zero data BIDs whose NID matches the requested row NID;
4. resolves those BIDs back to decoded payload blocks;
5. preserves ambiguity rather than selecting one candidate heuristically.

The result retains:

- matching SLBLOCK entry count;
- borrowed payload candidates;
- one bounded status.

Statuses are:

- `tc_row_payload_candidates_nid_missing`;
- `tc_row_payload_candidates_payload_missing`;
- `tc_row_payload_candidates_resolved`;
- `tc_row_payload_candidates_ambiguous`.

## Regression coverage

Tests cover:

- one matching entry and one resolved 208-byte payload;
- a matching entry whose payload is absent;
- two matching entries and two candidates remaining ambiguous;
- a declared SLBLOCK count larger than the available complete entries.

## Extraction impact

PQ69 does not alter extraction output, diagnostic serialization, or extraction volume. The validated public-fixture baseline remains:

- 14 TCINFO descriptors;
- 4 rows;
- 52-byte row width;
- 208-byte row payload;
- expected absolute offsets `0,52,104,156`;
- bitmap masks `11111011000000` for each row;
- 1 extracted message;
- 2 body payloads;
- 0 attachments.

The improvement is architectural: the production reporting path can now obtain the same payload candidate set required by the PQ68 bridge without reimplementing SLBLOCK traversal.

## Safety boundary

This increment does not:

- select one candidate when multiple candidates exist;
- serialize row payload bytes;
- invoke the PQ68 bridge from reporting;
- interpret property tags or bitmap bits;
- select or decode fixed-width row values;
- follow HID/HNID-backed values;
- alter message or attachment extraction.

## Proposed PQ70

PQ70 should integrate candidate resolution and the PQ68 bridge into `report_table_heaps`.

Requirements:

1. resolve payload candidates through `resolve_row_payload_candidates` immediately after `resolve_subnode_row_storage`;
2. pass borrowed payload byte slices to `build_transport_from_row_resolution`;
3. publish transport status for every subnode-backed row resolution;
4. publish address mode, row width, and absolute offsets only for validated transport;
5. serialize no payload bytes;
6. emit no partial metadata for unavailable or construction-failed transport;
7. preserve all descriptor, bitmap, layout, and extraction diagnostics;
8. add fixture-facing assertions for ordinal mode and offsets `0,52,104,156`;
9. continue to defer fixed-width descriptor selection and semantic interpretation.
