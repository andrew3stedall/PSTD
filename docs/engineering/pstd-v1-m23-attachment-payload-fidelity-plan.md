# PSTD v1 M23 Implementation Plan

## Implementation intent

M23 improves attachment fidelity without widening parser scope. The implementation preserves metadata already available from selected MAPI properties and compact attachment-table decoders, especially when a row is known but the byte payload is missing, empty, or deferred.

## Current foundation

- `AttachmentRecord` rows are emitted to `data/attachments.jsonl`.
- Extracted attachment payloads write raw bytes and hashes.
- Attachment subnode decoding already supports generic table contexts, `CATB`, and `CATW` compact attachment table rows.
- M20 and M21 already cover the focused `CATW` decoder and evidence classification.
- Before M23, missing payload rows could be collapsed to generic unavailable rows, losing row metadata.

## Implemented M23 slice

1. Attachment schema expansion:
   - Adds `declared_size_bytes`.
   - Adds `size_status`.
   - Adds `attachment_method`.
2. Metadata-only attachment rows:
   - `attachment_payloads_from_table` now returns extracted payloads and unavailable metadata records.
   - `attachment_payloads_from_subnode_blocks` aggregates unavailable metadata records across parsed tables.
   - The extraction engine appends unavailable metadata rows to `data/attachments.jsonl`.
3. Size fidelity:
   - Extracted rows are marked `size_matched`, `size_mismatch`, or `declared_size_absent`.
   - Missing rows are marked `payload_unavailable_declared_size_present` or `payload_unavailable_size_unknown`.
4. Embedded-message status:
   - Attachment method `5` is treated as an embedded-message attachment marker.
   - If bytes are unavailable, the row status becomes `embedded_message_payload_deferred`.
5. Compact table fidelity:
   - `CATB` and `CATW` rows with empty payloads now emit unavailable metadata records rather than counters only.

## Compatibility rules

| Input shape | Output behaviour |
|---|---|
| Attachment row has payload bytes | Emit extracted `AttachmentPayload` and attachment metadata row. |
| Attachment row has metadata but no payload bytes | Emit metadata-only `AttachmentRecord` with unavailable/deferred status. |
| Declared size equals payload size | `size_status=size_matched`. |
| Declared size differs from payload size | `size_status=size_mismatch`. |
| Payload exists with no declared size | `size_status=declared_size_absent`. |
| Payload missing with declared size | `size_status=payload_unavailable_declared_size_present`. |
| Payload missing and no declared size | `size_status=payload_unavailable_size_unknown`. |
| Attachment method indicates embedded message but bytes are unavailable | `extraction_status=embedded_message_payload_deferred`. |

## Definition of done

- M23 branch keeps CI green.
- Output contract documents new attachment fields.
- README, roadmap, PRD, project status, and changelog reflect M23 completion after merge.
- M24 remains focused on batch scale, performance, and corruption hardening.
