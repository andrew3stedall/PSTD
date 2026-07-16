# Vertical 32: Emit Tika heap-backed recipients

_Last reviewed: 16 July 2026._

## Objective

Emit the recipient rows stored inside the Table Context heaps of `tika-testPST.pst`, while attributing each table to its owning message and excluding the embedded-message recipient table from the outer attachment message.

## Root cause

The Tika message tables were already parsed, but their row matrix HNID (`0x80`) names an allocation in the same Heap-on-Node payload. The production projection only transported row matrices resolved through subnodes, so every heap-backed table reported a zero row width and emitted no recipients.

The attachment message also contains a recursive method-`5` embedded-message subtree. Selecting any recipient-like table found by the recursive probe would therefore mix ownership. The message root's direct Unicode SLBLOCK provides the required boundary: only direct NID type `0x12` data BIDs are eligible recipient tables for that message.

## Observable result

The fixture now emits eight structured recipients across seven messages:

| Evidence | Result |
|---|---:|
| Recipient rows | 8 |
| SMTP recipients | 6 |
| Raw/native recipients | 2 |
| Messages with one recipient | 6 |
| Messages with two recipients | 1 |

The native rows demonstrate the fail-closed address boundary:

- `msg_4bc9d7bd77cbe661` preserves `Hong-Thai Nguyen` and the complete `/o=ExchangeLabs/...` legacy Exchange distinguished name without inventing SMTP.
- `msg_c6163b9157944cc9` preserves display name `'lfcnassif@gmail.com'` and raw `PidTagEmailAddress` value `lfcnassif@gmail.com`; it is not relabelled as authoritative SMTP merely because the string resembles an Internet address.

The other six rows expose validated `PidTagSmtpAddress` values with `address_type = SMTP`, `smtp_address` populated, and `resolution_status = smtp_available`.

## Implementation

- Resolve non-subnode row HNIDs through the owning Table Context Heap-on-Node allocation.
- Reuse the existing row-layout, bitmap, transport, fixed-width, string, and complete-recipient projections for the resolved bytes.
- Generalise the row transport helpers so the same validated pipeline accepts either subnode-backed or heap-backed row slices.
- Parse only the message root's direct Unicode SLBLOCK recipient NIDs for production attribution.
- Require exactly one validated direct recipient projection per message; ambiguity still emits no partial records.
- Prevent the attachment-owner subnode tree from being projected twice when attachment presence is inferred from that same tree.

## Before versus after

| Measure | Before | After |
|---|---:|---:|
| Messages | 7 | 7 |
| Body records | 8 | 8 |
| Recipient records | 0 | 8 |
| Recipient JSONL bytes | 0 | 2,418 |
| Attachment records | 1 | 1 |
| Attachment payload files / bytes | 1 / 11,862 | 1 / 11,862 |
| EML files / bytes | 0 / 0 | 0 / 0 |
| Extraction TAR bytes | 164,352 | 202,752 |
| Total output bytes | 191,240 | 241,579 |

The existing DOCX filename, 11,862-byte payload, size mismatch evidence, SHA-256, ZIP CRC validation, and document text remain unchanged.

## Safety boundary

- Heap row resolution requires a valid heap HNID allocation and retains all existing row-width, row-reference, bitmap, and descriptor bounds.
- A message can select only a direct NID type `0x12` table from its root Unicode leaf SLBLOCK.
- Nested recipient tables, including the embedded message's BID `0x61c`, are not attributed to the outer message.
- Missing, malformed, multiple, partially populated, or row-misaligned candidates remain unavailable.
- Native address values are preserved as raw evidence; SMTP is emitted only from a validated `PidTagSmtpAddress` projection.

## Validation

Focused tests cover heap-backed row storage, shared row transport, direct recipient-table ownership, nested-table exclusion, and end-to-end complete record projection. The permanent Tika workflow asserts all eight exact records, including stable keys, message ownership, row order, address type, raw address, SMTP availability, and the legacy Exchange distinguished name. It also retains the exact attachment and aggregate byte contracts.

## Next vertical milestone

Validate the remaining Date/header requirements for `msg_c6163b9157944cc9`, then assemble its first deterministic `multipart/mixed` EML with plain text, HTML, and the existing DOCX payload. Keep the method-`5` embedded message as a separate later boundary.
