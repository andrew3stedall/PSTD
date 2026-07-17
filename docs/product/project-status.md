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
| Embedded message | Validated through Vertical 34 / PR #455 | One method-`5` PtypObject resolves to a separate linked message with one directly owned recipient, a 23-byte text body, and four preserved raw HTML-property bytes; no child evidence is projected onto the parent. |
| Downstream systems | Parked | Snowflake, UI, search, analytics, semantic search, and graph work remain out of scope. |

## Tika Vertical 34 evidence

| Metric | Vertical 33 baseline | PR #455 result |
|---|---:|---:|
| Messages | 7 | 8 |
| Body records | 8 | 10 |
| Body payload files / bytes | 6 / 252 | 8 / 279 |
| Recipient records | 8 | 9 |
| Recipient JSONL bytes | 2,418 | 2,708 |
| Attachment records | 1 | 2 |
| Attachment payload files / bytes | 1 / 11,862 | 1 / 11,862 |
| EML files / bytes | 1 / 17,035 | 1 / 17,035 |
| Extraction TAR bytes | 202,752 | 227,840 |
| Total extraction-output bytes | 241,579 | 272,884 |

The method-`5` Property Context now preserves its object HNID, validates the exact eight-byte PtypObject `Nid + ulSize` allocation, requires a normal-message NID, and resolves that NID exactly once within the outer message's loaded subnode scope. The linked child `msg_0ff529af59d373d5` owns its own recipient and body records. The parent keeps its original recipient, DOCX ordinal/key/path, and 17,035-byte EML.

The child's four-byte `PidTagHtml` evidence is `7f 83 00 00`. It remains a raw body artefact and is not promoted to MIME HTML. The method-`5` attachment is metadata-only and links to the child with `embedded_message_key`; no empty EML payload is written at its archive path.

## Latest completed work

PR #455, **Recover Tika embedded message as a separate object**, preserves PtypObject HNIDs before heap dereference, decodes the specification-defined object wrapper, requires one unambiguous child NID in the parent message scope, isolates the child's subnode subtree, and reuses the existing message/body/direct-recipient projections. Ambiguous, missing, malformed, wrong-type, or duplicate references remain unavailable.

## Next evidence-based milestone

Emit a deterministic plain-text-only EML for the recovered child. Its validated sender, recipient, subject, received-time Date evidence, Message-ID, and 23-byte UTF-8 body are available, but the current attachmentless EML path requires a validated HTML alternative. The raw four-byte HTML property must remain excluded. Materialising that child EML as the parent method-`5` attachment payload remains a later explicit boundary.

## Validation expectations

Every extraction PR must pass formatting, clippy with warnings denied, all Rust tests, CLI checks, Python wrapper checks, Docker build, approved fixture workflows, and exact artifact review. Unsupported or ambiguous candidates must remain unavailable rather than producing partial records.

## Risk statement

The new result is evidence for one approved Unicode PST, not broad PST compatibility. The sender's Exchange distinguished name is preserved but not SMTP-resolved. ANSI files, uncommon or corrupt layouts, nested embedded attachments, contacts/distribution lists, and many MAPI property combinations remain incomplete.
