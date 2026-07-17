# PSTD Project Status

_Last reviewed: 17 July 2026._

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
| Tika attachment EML | Validated through Vertical 33 / PR #454 | One deterministic 17,035-byte `multipart/mixed` EML contains the 22-byte UTF-8 plain-text body and exact 11,862-byte DOCX payload. The invalid four-byte HTML value is excluded. |
| Embedded message | Deferred | The method-`5` embedded-message subtree is deliberately excluded from the outer message's recipient ownership and awaits a separate extraction path. |
| Downstream systems | Parked | Snowflake, UI, search, analytics, semantic search, and graph work remain out of scope. |

## Tika Vertical 33 evidence

| Metric | Vertical 32 baseline | PR #454 result |
|---|---:|---:|
| Messages | 7 | 7 |
| Body records | 8 | 8 |
| Recipient records | 8 | 8 |
| Recipient JSONL bytes | 2,418 | 2,418 |
| Attachment records | 1 | 1 |
| Attachment payload files / bytes | 1 / 11,862 | 1 / 11,862 |
| EML files / bytes | 0 / 0 | 1 / 17,035 |
| Extraction TAR bytes | 202,752 | 202,752 |
| Total extraction-output bytes | 241,579 | 241,579 |

The EML Date is `Thu, 26 Nov 2020 22:18:00 +0000`, derived from the attachment owner's validated `PidTagMessageDeliveryTime` FILETIME because no transport Date or submit time is available. The native Exchange sender and raw recipient address remain preserved without SMTP invention. The EML adds no changes to structured extraction or archive bytes.

## Latest completed work

PR #454, **Emit attachment-bearing Tika EML**, groups validated attachment payloads by message and ordinal, requires bounded Date evidence, emits a plain-text `multipart/mixed` body when HTML is unusable, and includes only non-empty by-value payloads whose length and SHA-256 match their records. The permanent fixture parses the MIME tree and verifies the decoded DOCX bytes exactly.

## Next evidence-based milestone

Recover the Tika method-`5` embedded message as a separate message object and attachment path. Preserve the outer message's direct ownership boundary and do not attribute the embedded message's recipients, body, or identifiers to its parent.

## Validation expectations

Every extraction PR must pass formatting, clippy with warnings denied, all Rust tests, CLI checks, Python wrapper checks, Docker build, approved fixture workflows, and exact artifact review. Unsupported or ambiguous candidates must remain unavailable rather than producing partial records.

## Risk statement

The new result is evidence for one approved Unicode PST, not broad PST compatibility. The sender's Exchange distinguished name is preserved but not SMTP-resolved. ANSI files, uncommon or corrupt layouts, embedded messages, contacts/distribution lists, and many MAPI property combinations remain incomplete.
