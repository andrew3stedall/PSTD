# PSTD v1 M10 Implementation Plan

## Implementation intent

M10 connects earlier parser foundations into concrete payload wiring helpers without claiming full corpus-grade recursive decoding.

## Current foundation

- M8 added bounded BBT and node-index traversal.
- M8 added table and property parse reports.
- M9 added parser limits, payload block loading, body/attachment payload builders, and subnode-reference reporting.
- M1-M9 CI is green.

## Implemented M10 slice

1. Node payload wiring:
   - Loads a node data block through `BbtIndex` and `PstByteReader`.
   - Parses the loaded bytes as a BTH property map.
   - Converts the BTH map into a `PropertyContext`.
   - Emits a `NodePayloadReport` with node ID, data block ID, payload size, property count, and status.
2. Subnode decode planning:
   - Converts discovered subnode references into bounded decode plans.
   - Applies `ParserLimits::max_subnode_depth`.
   - Emits explicit depth-limit status.
3. Attachment table wiring:
   - Converts table rows into `PropertyContext` values.
   - Reuses M9 attachment payload builders.
   - Emits an `AttachmentTableWiringReport` with row counts and missing payload counts.

## Operational behaviour

- M10 makes node-to-property and table-to-attachment wiring testable independently from full PST corpus complexity.
- Recursive subnode binary format decoding remains intentionally separate.
- Attachment rows without payload bytes remain explicit rather than silent.

## Remaining work

- Decode recursive subnode binary structures.
- Integrate node payload wiring into the main extraction path once parser evidence is sufficient.
- Integrate attachment table wiring into the main archive writer.
- Add broader synthetic/public fixture coverage.
- Add real-world corpus validation without committing private PST data.

## Safety and privacy

Do not add private PST files, extracted content, or batch outputs as fixtures. Use synthetic or clearly public fixture data only.

## Definition of done

- M10 branch keeps CI green.
- Node payload wiring is test-covered.
- Subnode decode planning is test-covered.
- Attachment table wiring is test-covered.
- Docs describe remaining limitations accurately.
