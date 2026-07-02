# PSTD v1 M13 Implementation Plan

## Implementation intent

M13 does not attempt broad PST parser expansion. It improves confidence in the existing payload path by adding synthetic fixture-style coverage and compatibility diagnostics for layouts that are not yet fully decoded.

## Current foundation

- M11 writes payload bytes into TAR archives when helpers produce payloads.
- M12 connects bounded subnode root-block loading to attachment-table parsing and attachment payload construction.
- Existing tests cover basic body and attachment payload construction.

## Implemented M13 slice

1. Attachment subnode compatibility diagnostics:
   - Adds parse-error offsets to `AttachmentSubnodeWiringReport`.
   - Adds parse-error reasons to `AttachmentSubnodeWiringReport`.
   - Adds parsed table statuses to `AttachmentSubnodeWiringReport`.
2. Synthetic attachment fixture expansion:
   - Adds mixed-block coverage for one extracted payload, one unparseable block, and one parsed table without payload bytes.
   - Verifies partial payload status and diagnostic counts.
3. Synthetic body fixture expansion:
   - Adds all-supported-body coverage for text, HTML, and RTF payloads.
   - Verifies archive paths and bytes for the supported payload types.

## Status behaviour

M13 preserves M12 status values and adds richer report fields for compatibility triage:

- `parse_error_offsets`
- `parse_error_reasons`
- `table_statuses`

This is intended to help future fixture validation distinguish parser gaps from genuinely absent payloads.

## Remaining work

- Add licensed/public PST fixture coverage where legally safe.
- Add sanitized private fixture strategy if required.
- Decode recursive child-subnode layouts beyond M12 root-block handling.
- Add compatibility classification for additional PST table/subnode variants as they are observed.

## Definition of done

- M13 branch keeps CI green.
- Synthetic payload coverage includes body and attachment paths.
- Compatibility diagnostics expose enough detail for future fixture triage.
- Project status and changelog document the M13 slice and remaining limitations.
