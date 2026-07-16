# Vertical 31: Emit the DOCX attachment payload

_Last reviewed: 16 July 2026._

## Objective

Decode the validated Unicode XBLOCK rooted at BID `0x632` for the Tika fixture attachment and emit the exact by-value `attachment.docx` payload without treating internal NDB blocks as file bytes.

## Evidence correction

The real fixture distinguishes two sizes:

- `PidTagAttachSize`: 15,503 bytes;
- XBLOCK `lcbTotal`: 11,862 bytes.

The first value is retained as source attachment metadata. It is not used as the data-tree byte count. The XBLOCK total and the exact concatenated child bytes are the authoritative payload length for this milestone. This replaces the earlier assumption that both values had to match.

## Acceptance boundary

The milestone must:

- preserve the owning message `msg_c6163b9157944cc9` and attachment metadata already validated in Verticals 29 and 30;
- require an internal Unicode XBLOCK with `btype = 0x01` and `cLevel = 0x01`;
- read its ordered 64-bit child BIDs through the existing bounded BBT and payload loader;
- reject zero, duplicate, internal, missing, truncated, or over-limit children;
- require the assembled payload to equal the XBLOCK `lcbTotal` of 11,862 bytes;
- preserve the differing 15,503-byte `PidTagAttachSize` value and report the size relationship explicitly;
- require the DOCX ZIP signature `50 4b 03 04` before publication;
- emit one deterministic attachment payload, SHA-256 checksum, archive path, and structured record;
- suppress unrelated attachment-table fallback rows once the validated filename-bearing Property Context path is selected;
- keep the method-`5` embedded-message context outside this vertical.

Exact fixture counts, output-byte deltas, checksum, child BIDs, DOCX container validation, and the following blocker will be recorded from the completed acceptance artifact before merge.
