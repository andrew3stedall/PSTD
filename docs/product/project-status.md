# PSTD Project Status

_Last reviewed: 18 July 2026._

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
| Embedded child EML | Validated through Vertical 35 / PR #457 | The linked child emits one deterministic 453-byte single-part `text/plain` EML with exact headers, CRLF body assembly, and SHA-256; raw HTML bytes remain excluded, and unrelated plain-only messages remain unavailable. |
| Downstream systems | Parked | Snowflake, UI, search, analytics, semantic search, and graph work remain out of scope. |

## Tika Vertical 35 evidence

| Metric | Vertical 34 baseline | PR #457 result |
|---|---:|---:|
| Messages | 8 | 8 |
| Body records | 10 | 10 |
| Body payload files / bytes | 8 / 279 | 8 / 279 |
| Recipient records | 9 | 9 |
| Recipient JSONL bytes | 2,708 | 2,708 |
| Attachment records | 2 | 2 |
| Attachment payload files / bytes | 1 / 11,862 | 1 / 11,862 |
| EML files / bytes | 1 / 17,035 | 2 / 17,488 |
| Extraction TAR bytes | 227,840 | 227,840 |
| Total extraction-output bytes | 272,884 | 272,884 |

The linked child `msg_0ff529af59d373d5` now emits a deterministic 453-byte single-part `text/plain` EML with SHA-256 `86ffe5567da7aa505b8be16400889170ca583fd247cc0758f00a43c2a8a99420`. Its From, To, Subject, Date, Message-ID, MIME-Version, transfer encoding, CRLF line endings, and exact 23-byte body evidence are locked by the fixture workflow. The parent keeps its original recipient, DOCX ordinal/key/path, and 17,035-byte EML.

Plain-text-only admission is authorised from `AttachmentRecord.embedded_message_key`, not from payload availability. Ordinary top-level messages without validated HTML remain unavailable. The child's four-byte `PidTagHtml` evidence `7f 83 00 00` remains a raw body artefact and is absent from MIME output. The method-`5` attachment is still metadata-only and has no published payload file.

## Latest completed work

PR #457, **Emit recovered child as plain-text EML**, adds a policy-gated attachmentless EML path for message keys referenced by authoritative attachment metadata. It emits one exact child EML, retains the parent's MIME and DOCX bytes, and preserves fail-closed behaviour for unrelated plain-only records. Missing headers, invalid UTF-8, boundary collisions, ambiguous links, and unsupported candidates remain unavailable.

## Next evidence-based milestone

Materialise the exact 453-byte child EML as the method-`5` attachment payload. The new payload must use `message/rfc822`, retain explicit parent-child ownership, publish deterministic path/length/SHA-256 evidence, and replace the current metadata-only boundary without writing an empty placeholder. Nested recursion and outer-parent MIME inclusion remain separate unless independently validated in the same fixture slice.

## Validation expectations

Every extraction PR must pass formatting, clippy with warnings denied, all Rust tests, CLI checks, Python wrapper checks, Docker build, approved fixture workflows, and exact artifact review. Unsupported or ambiguous candidates must remain unavailable rather than producing partial records.

## Risk statement

The new result is evidence for one approved Unicode PST, not broad PST compatibility. The sender's Exchange distinguished name is preserved but not SMTP-resolved. ANSI files, uncommon or corrupt layouts, nested embedded attachments, contacts/distribution lists, and many MAPI property combinations remain incomplete.
