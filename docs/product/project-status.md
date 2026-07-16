# PSTD Project Status

_Last reviewed: 16 July 2026._

## Purpose

Provide the authoritative view of the merged extraction baseline and the next evidence-led boundary.

## Current implementation state

| Area | Current state | Evidence and limitations |
|---|---|---|
| Product foundation | Complete through M25 | Rust CLI, Python wrapper, Docker packaging, structured TAR/JSONL output, batch/resume, diagnostics, and operator guidance. |
| Bounded PST parser | Validated foundation through PQ74 | Header, BBT/NBT, blocks, subnodes, Heap-on-Node, BTH, Property Context, Table Context, row transport, and supported MAPI values with explicit limits. |
| Original public fixture | Material readable-email path | One message, four structured recipients, text and recovered HTML, and one deterministic 956-byte EML. |
| Tika DOCX attachment | Validated through Vertical 31 / PR #450 | One `attachment.docx` payload: 11,862 bytes, valid ZIP/CRC, expected document text, and preserved 15,503-byte source size metadata. |
| Tika recipients | Validated through Vertical 32 / PR #452 | Eight directly attributed recipients across seven messages: six SMTP rows and two raw/native rows, including a full legacy Exchange distinguished name. |
| Tika attachment EML | Not yet emitted | The attachment message now has body, recipient, and DOCX evidence; Date/required-header validation and multipart assembly remain. |
| Embedded message | Deferred | The method-`5` embedded-message subtree is deliberately excluded from the outer message's recipient ownership and awaits a separate extraction path. |
| Downstream systems | Parked | Snowflake, UI, search, analytics, semantic search, and graph work remain out of scope. |

## Tika Vertical 32 evidence

| Metric | Before | PR #452 result |
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

The eight rows match the validated row-index inventory: six messages have one row and one message has two. Six expose `PidTagSmtpAddress`. The other two preserve raw `PidTagEmailAddress` values without guessing SMTP: the attachment owner and a legacy Exchange distinguished-name recipient.

## Latest completed work

PR #452, **Emit heap-backed Tika recipient rows**, resolved row HNID `0x80` from the owning Table Context heap, reused the existing bounded row and recipient projections, selected only direct root-SLBLOCK recipient-table BIDs, and prevented the attachment owner's subnode tree from being projected twice. It merged as `1ecc3e56c1d441ff95148618f08d07d7b18ab559` after all exact-head CI and fixture workflows passed.

## Next evidence-based milestone

Validate the remaining Date and required headers for `msg_c6163b9157944cc9`, then assemble one deterministic `multipart/mixed` EML containing the validated plain-text/HTML body and unchanged DOCX payload. Do not fold the method-`5` embedded message into that work.

## Validation expectations

Every extraction PR must pass formatting, clippy with warnings denied, all Rust tests, CLI checks, Python wrapper checks, Docker build, approved fixture workflows, and exact artifact review. Unsupported or ambiguous candidates must remain unavailable rather than producing partial records.

## Risk statement

The new result is evidence for one approved Unicode PST, not broad PST compatibility. ANSI files, uncommon or corrupt layouts, embedded messages, contacts/distribution lists, and many MAPI property combinations remain incomplete.
