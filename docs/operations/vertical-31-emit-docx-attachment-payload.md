# Vertical 31: Emit the DOCX attachment payload

_Last reviewed: 16 July 2026._

## Objective

Decode the validated Unicode XBLOCK rooted at BID `0x632` for the Tika fixture attachment and emit the exact by-value `attachment.docx` payload without treating internal NDB blocks as file bytes.

## Observable result

PSTD now emits one real attachment payload for the known attachment-owning message:

```text
message_key:            msg_c6163b9157944cc9
message_node_id:        node_2000e4
filename:               attachment.docx
archive_path:           attachments/msg_c6163b9157944cc9/att_0695091e19397627_attachment.docx
data NID:               0x0000833f
data-tree root BID:     0x632
XBLOCK child blocks:    2
payload bytes:          11,862
PidTagAttachSize:       15,503
SHA-256:                0c87a742c970907d3b08c73e7834768abadd00fe4f4995a7dd98a206d4c494c0
ZIP signature:          50 4b 03 04
```

The generated file is a valid DOCX ZIP container. All members pass CRC validation, `word/document.xml` is readable, and it contains the upstream expected text `This is a docx attachment.`

## Evidence correction

The real fixture distinguishes two sizes:

- `PidTagAttachSize`: 15,503 bytes;
- XBLOCK `lcbTotal`: 11,862 bytes.

The first value is retained as source attachment metadata. It is not used as the data-tree byte count. The XBLOCK total and the exact concatenated child bytes are the authoritative payload length for this milestone. The structured record therefore reports `size_bytes = 11862`, `declared_size_bytes = 15503`, and `size_status = size_mismatch` without truncating, padding, or inventing bytes.

## Safety boundary

The decoder:

- accepts only an internal Unicode XBLOCK with `btype = 0x01` and `cLevel = 0x01`;
- parses the declared 16-bit child count and 32-bit `lcbTotal`;
- resolves ordered 64-bit child BIDs through the existing bounded BBT and payload loader;
- rejects zero, duplicate, internal, missing, truncated, overflowing, or over-limit child references;
- requires the concatenated child data to equal XBLOCK `lcbTotal` exactly;
- requires the DOCX ZIP signature before creating an `AttachmentPayload`;
- preserves the differing `PidTagAttachSize` value instead of forcing the payload to that size;
- suppresses unrelated attachment-table fallback rows after the validated filename-bearing Property Context path is selected;
- leaves the separate method-`5` embedded-message context deferred.

## Before versus after

| Measure | Before | After |
|---|---:|---:|
| Messages | 7 | 7 |
| Message JSONL bytes | 21,435 | 21,521 |
| Body records | 8 | 8 |
| Body payload files / bytes | 6 / 252 | 6 / 252 |
| Recipient records | 0 | 0 |
| Attachment records | 1 | 1 |
| Attachment JSONL bytes | 648 | 643 |
| Attachment payload files / bytes | 0 / 0 | 1 / 11,862 |
| EML files / bytes | 0 / 0 | 0 / 0 |
| Extraction TAR bytes | 126,464 | 164,352 |
| Total output bytes | 147,692 | 191,240 |

One message and one attachment record are affected. The owning message now reports `attachment_count = 1` and `extraction_status = metadata_and_payload`.

## Validation

The permanent Tika fixture gate asserts:

- the exact message, filename, method, metadata size, payload size, checksum, archive path, and extraction status;
- exactly one attachment record and one attachment payload file;
- the `50 4b 03 04` signature;
- successful ZIP CRC validation;
- the expected text in `word/document.xml`;
- seven messages, eight body records, six body payload files totalling 252 bytes, zero recipients, and zero EML files;
- exact JSONL, TAR, and total-output byte counts.

The original readable EML and readable RTF/HTML fixture gates remain unchanged.

## Next vertical milestone

Extract the first validated recipient row owned by `msg_c6163b9157944cc9`. Preserve its role, display name, address type, and raw legacy Exchange address, and emit SMTP only when uniquely supported by validated same-PST evidence. Do not assemble `multipart/mixed` EML until recipient and required header evidence is independently complete.
