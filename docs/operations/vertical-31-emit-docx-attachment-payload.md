# Vertical 31: Emit the DOCX attachment payload

_Last reviewed: 16 July 2026._

## Objective

Decode the validated Unicode XBLOCK rooted at BID `0x632` for the Tika fixture attachment and emit the exact by-value `attachment.docx` payload without treating internal NDB blocks as file bytes.

## Acceptance boundary

The milestone must:

- preserve the owning message `msg_c6163b9157944cc9` and attachment metadata already validated in Verticals 29 and 30;
- require an internal Unicode XBLOCK with `btype = 0x01` and `cLevel = 0x01`;
- read its ordered 64-bit child BIDs through the existing bounded BBT and payload loader;
- reject zero, duplicate, internal, missing, truncated, or over-limit children;
- require the XBLOCK total and assembled payload to equal the declared attachment size of 15,503 bytes;
- require the DOCX ZIP signature `50 4b 03 04` before publication;
- emit one deterministic attachment payload, SHA-256 checksum, archive path, and structured record;
- keep the method-`5` embedded-message context outside this vertical.

Exact fixture counts, output-byte deltas, checksum, child BIDs, and the following blocker will be recorded from the completed acceptance artifact before merge.
