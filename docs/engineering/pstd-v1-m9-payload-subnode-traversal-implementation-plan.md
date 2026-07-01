# PSTD v1 M9 Implementation Plan

## Implementation intent

M9 adds safe payload and subnode foundations without claiming full real-world payload extraction. The slice is designed to make later folder, recipient, body, and attachment traversal work easier to implement and validate.

## Current foundation

- M8 added bounded BBT and node-index traversal.
- M8 added table and property parse reports.
- M5 added deterministic body and attachment record helpers.
- M1-M8 CI is green.

## Implemented M9 slice

1. Parser limits:
   - `max_btree_pages`.
   - `max_block_bytes`.
   - `max_subnode_depth`.
2. BBT/NBT traversal limit APIs:
   - Existing `load_root` APIs still use defaults.
   - New `load_root_with_limits` APIs accept explicit `ParserLimits`.
3. Payload block loading:
   - Resolves a block through `BbtIndex::lookup`.
   - Enforces `max_block_bytes`.
   - Reads the block through the existing bounded `load_block` path.
4. Body payload construction:
   - Builds text body payloads from `PR_BODY`.
   - Builds HTML body payloads from `PR_HTML`.
   - Builds RTF body payloads from `PR_RTF_COMPRESSED`.
5. Attachment payload construction:
   - Builds attachment payloads from `PR_ATTACH_DATA_BIN`.
   - Preserves long filename, MIME tag, content ID, and inline/hidden indicators.
6. Subnode-reference reporting:
   - Reports node-index entries that reference subnode blocks.
   - Keeps recursive decoding out of this slice.

## Operational behaviour

- Existing extraction remains conservative when real payload property contexts are unavailable.
- When a property context is available, M9 helpers can produce body and attachment payload records plus bytes.
- Payload block loading is explicitly capped to avoid unexpectedly large reads.
- Subnode references are observable before recursive traversal is implemented.

## Remaining work

- Decode subnode block structures recursively.
- Connect real message data blocks into property contexts.
- Connect real attachment table rows and attachment subnodes into attachment payload extraction.
- Add broader synthetic/public fixtures for payload and subnode cases.
- Add CLI/config exposure for parser limits if operational testing shows it is needed.

## Safety and privacy

Do not add private PST files, extracted content, or batch outputs as fixtures. Use synthetic or clearly public fixture data only.

## Definition of done

- M9 branch keeps CI green.
- Parser limits are covered by tests.
- Payload block loading is covered by tests.
- Body and attachment property-context builders are covered by tests.
- Subnode-reference reporting is covered by tests.
- Docs describe remaining limitations accurately.
