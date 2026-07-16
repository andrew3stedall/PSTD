# Vertical 30: Resolve the DOCX attachment data reference

_Last reviewed: 16 July 2026._

## Objective

Resolve the validated four-byte `PidTagAttachDataBinary` HNID for the Tika `attachment.docx` record through the owning message's Unicode SLBLOCK, without treating an internal block as attachment bytes.

## Observable result

For message `msg_c6163b9157944cc9` (`node_2000e4`), PSTD now resolves:

```text
PidTagAttachDataBinary raw value: 3f830000
attachment data NID:             0x0000833f
resolved data BID:               0x632
filename:                        attachment.docx
declared size:                   15,503 bytes
attachment method:               1 (ATTACH_BY_VALUE)
payload bytes emitted:           0
```

The attachment record status is now:

```text
attachment_metadata_extracted_payload_subnode_reference_resolved; data_nid=0x0000833f; data_bid=0x632
```

One message and one attachment record are affected.

## Why this is a vertical extraction result

The previous milestone could identify the attachment but stopped at an opaque four-byte reference. This milestone exposes the exact PST subnode identity and concrete data-block identity for the real public-fixture attachment. That mapping is new observable fixture data and is the immediate prerequisite for decoding the internal XBLOCK data tree.

It does not add a generic transport wrapper or report-only abstraction. The resolver is used directly while constructing the existing attachment record.

## Fail-closed boundary

The resolver accepts a mapping only when all of the following hold:

- `PidTagAttachDataBinary` contains exactly four raw bytes;
- the HNID low five bits identify a subnode NID rather than a heap HID;
- the source block is a complete Unicode leaf SLBLOCK;
- the SLBLOCK contains an exact NID match and a non-zero data BID;
- that data BID is present in the bounded recursive subnode block set already loaded for the message.

The referenced BID is internal. PSTD therefore records the mapping but does not emit those internal bytes as DOCX content.

## Before versus after

| Measure | Before | After |
|---|---:|---:|
| Messages | 7 | 7 |
| Message JSONL bytes | 21,435 | 21,435 |
| Body records | 8 | 8 |
| Body payload files / bytes | 6 / 252 | 6 / 252 |
| Recipient records | 0 | 0 |
| Attachment records | 1 | 1 |
| Attachment JSONL bytes | 605 | 648 |
| Attachment payload files / bytes | 0 / 0 | 0 / 0 |
| EML files / bytes | 0 / 0 | 0 / 0 |
| Extraction TAR bytes | 126,464 | 126,464 |
| Total output bytes | 147,692 | 147,692 |

The 43-byte structured-output increase is the exact resolved `data_nid` and `data_bid` status. TAR block padding means archive and total output byte counts remain unchanged.

## Tests

Focused regression tests cover:

- exact NID-to-BID resolution;
- requirement that the resolved BID is actually loaded;
- mismatched NID rejection;
- truncated SLBLOCK rejection;
- preservation of existing blank, incomplete, and wrongly typed attachment-context rejection.

The permanent Tika fixture workflow asserts the exact NID, BID, attachment metadata, counts, and output byte totals.

## Remaining blocker

BID `0x632` is an internal NDB data-tree block, not direct DOCX bytes. PSTD must decode its XBLOCK child references, concatenate the external data blocks in order, require exactly 15,503 bytes, and verify the DOCX ZIP signature before emitting a payload.

## Next vertical milestone

Decode the `0x632` XBLOCK tree and emit exactly one 15,503-byte `attachment.docx` payload with a deterministic SHA-256 checksum and archive path. Embedded-message method `5` remains outside that milestone.
