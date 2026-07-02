# PSTD v1 M12 Implementation Plan

## Implementation intent

M12 connects subnode references to attachment-table parsing in the main processing path. The implementation remains bounded and conservative: it loads only the referenced subnode root block through existing BBT-backed payload loading, then attempts table parsing and attachment payload construction from that loaded block.

## Current foundation

- M9 added parser limits and BBT-backed payload loading.
- M10 added attachment table row to payload conversion.
- M11 writes payload bytes to TAR archives when extraction produces payloads.

## Implemented M12 slice

1. Bounded subnode block loading:
   - Adds `load_bounded_subnode_blocks`.
   - Enforces `ParserLimits::max_subnode_depth`.
   - Emits `SubnodeDecodeReport` with decoded counts, failure counts, byte counts, and status.
2. Attachment table parsing from subnode blocks:
   - Adds `attachment_payloads_from_subnode_blocks`.
   - Attempts `TableContext::parse_with_report` on loaded subnode blocks.
   - Converts parsed table rows through existing `attachment_payloads_from_table`.
   - Reports parsed table count, parse errors, row count, payload count, and missing payload count.
3. Main processing-path integration:
   - When message metadata says attachments exist, M12 checks for a matching subnode reference.
   - If a reference exists, M12 loads the bounded subnode block and attempts attachment-table extraction.
   - If payloads are produced, attachment records and payload bytes flow through the existing M11 archive path.
   - If payloads are not produced, M12 emits explicit unavailable attachment records and status messages.

## Status behaviour

M12 distinguishes these cases:

- `attachment_subnode_reference_absent`
- `subnode_depth_limit_exceeded`
- `subnode_root_block_loaded`
- `subnode_root_block_unavailable`
- `attachment_subnode_payloads_wired`
- `attachment_subnode_payloads_partially_wired`
- `attachment_subnode_tables_without_payloads`
- `attachment_subnode_tables_unavailable`

## Remaining work

- Decode recursive child-subnode structures beyond the currently referenced root block.
- Validate against broader public fixtures.
- Add richer PST-aware attachment table identification once deeper LTP parsing exists.
- Avoid committing private PST files.

## Definition of done

- M12 branch keeps CI green.
- Subnode loading and attachment subnode table conversion have synthetic tests.
- Main processing path attempts attachment extraction via subnode references.
- Unsupported paths remain explicit in JSONL/status output.
