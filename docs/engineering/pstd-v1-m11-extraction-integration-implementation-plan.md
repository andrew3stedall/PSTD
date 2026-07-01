# PSTD v1 M11 Implementation Plan

## Implementation intent

M11 moves from isolated payload wiring helpers to main extraction-path integration. The implementation remains conservative: it attempts node payload loading where the current parser can resolve data blocks, emits payload files when available, and records explicit unavailable rows where deeper parser support is still missing.

## Current foundation

- M8 added bounded BBT and node-index traversal.
- M9 added parser limits and BBT-backed payload block loading.
- M10 added node payload-to-property context wiring and attachment table wiring helpers.
- M1-M10 CI is green.

## Implemented M11 slice

1. Extraction-path parser limits:
   - Main metadata extraction now uses limit-aware BBT/NBT loading.
2. Node payload integration:
   - Node-index candidates are passed through node payload-to-property context loading.
   - Successful property contexts produce message metadata through the existing `message_from_properties` path.
   - Failed property context loading remains recoverable and emits candidate metadata plus an explicit status row.
3. Body payload integration:
   - Body payloads are generated from loaded property contexts when `PR_BODY`, `PR_HTML`, or `PR_RTF_COMPRESSED` are present.
   - Body metadata rows are emitted to `data/bodies.jsonl`.
   - Body bytes are written under `bodies/` in the TAR archive.
4. Attachment payload writer integration:
   - The archive writer now writes attachment payload bytes if metadata extraction produces attachment payloads.
   - Full real attachment-table extraction remains a later parser-depth slice.
5. Manifest integration:
   - Extracted body and attachment payloads receive per-payload manifest rows with archive paths, sizes, and hashes.

## Operational behaviour

- M11 does not silently drop payload paths that remain unsupported.
- Node-level parser failures are recoverable by default and are represented in status output.
- Payload bytes are only written when helper wiring produces explicit payload byte buffers.

## Remaining work

- Decode recursive subnode binary structures.
- Connect real attachment tables into the main extraction path.
- Connect table-derived attachment payloads into metadata extraction once real table traversal is available.
- Add broader synthetic/public fixture coverage for real body and attachment extraction.
- Continue avoiding private PST fixture commits.

## Definition of done

- M11 branch keeps CI green.
- Extraction path attempts node payload loading.
- Body payloads are written to TAR when available.
- Unavailable body paths are explicit.
- Docs describe remaining parser limits accurately.
