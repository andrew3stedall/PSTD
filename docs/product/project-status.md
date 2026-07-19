# PSTD Project Status

_Last reviewed: 20 July 2026._

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
| Embedded message | Validated through Vertical 34 / PR #455 | One method-`5` PtypObject resolves to a separate linked message with one directly owned recipient, a 23-byte text body, and an explicit unavailable HTML form for its four-byte property locator; no child evidence is projected onto the parent. |
| Embedded child EML | Validated through Vertical 35 / PR #457 | The linked child emits one deterministic 453-byte single-part `text/plain` EML with exact headers, CRLF body assembly, and SHA-256; raw HTML bytes remain excluded, and unrelated plain-only messages remain unavailable. |
| Method-5 child payload | Validated through Vertical 36 / PR #461 | The same exact 453 bytes now publish at the existing method-`5` archive path as `message/rfc822`; key, ordinal, owner and child link remain stable, with fail-closed duplicate, mismatch and nesting rejection. |
| Tika folder/message ownership | Validated through Vertical 37 / PR #464 | All eight folder records are exact; seven top-level messages resolve once from `node_802e` row keys to `/Début du fichier de données Outlook`; the linked embedded child remains isolated. |
| Independent body forms | Validated through Vertical 38 / PR #470 | Four-byte Property Context body locators remain explicit unavailable forms; the body-types fixture selects its valid 37-byte plain body, and the Tika parent/child retain plain text without materializing invalid HTML. |
| ANSI header diagnostics | Validated through Vertical 39 / PR #473 | Versions 14 and 15 use variant-correct 32-bit NBT/BBT root offsets and the ANSI crypt-method location. Values are diagnostic only; ANSI roots cannot authorize traversal or extraction. Synthetic tests prevent adjacent-byte contamination. |
| Downstream systems | Parked | Snowflake, UI, search, analytics, semantic search, and graph work remain out of scope. |

## Tika Vertical 36 evidence

| Metric | Vertical 35 baseline | PR #461 result |
|---|---:|---:|
| Messages | 8 | 8 |
| Body records | 10 | 10 |
| Body payload files / bytes | 8 / 279 | 8 / 279 |
| Recipient records | 9 | 9 |
| Attachment records | 2 | 2 |
| Attachment payload files / bytes | 1 / 11,862 | 2 / 12,315 |
| Attachment JSONL bytes | 1,358 | 1,240 |
| EML files / bytes | 2 / 17,488 | 2 / 17,488 |
| Extraction TAR bytes | 227,840 | 228,864 |
| Total extraction-output bytes | 272,884 | 273,908 |

The method-`5` record `att_a9c94a13d70f1cb3` now publishes a 453-byte `message/rfc822` payload with SHA-256 `86ffe5567da7aa505b8be16400889170ca583fd247cc0758f00a43c2a8a99420` at its existing archive path. Those bytes are identical to standalone child `msg_0ff529af59d373d5.eml`. The parent message key, attachment ordinal, filename, archive path and `embedded_message_key` remain stable.

The shared EML builder is used by both output paths. Missing or mismatched links, duplicate child references, nested child ownership, ambiguous text bodies, unsafe headers and invalid EML evidence remain unavailable. Parent MIME assembly continues to admit only supported method-`1` by-value payloads, preserving the exact 17,035-byte parent EML and 11,862-byte DOCX.

## Tika Vertical 37 evidence

| Metric | Vertical 36 baseline | PR #464 result |
|---|---:|---:|
| Folders | 8 observed | 8 exact |
| Top-level messages with exact physical owner | 0 | 7 |
| Embedded children kept outside physical ownership inference | 1 | 1 |
| Messages | 8 | 8 |
| Recipient records | 9 | 9 |
| Body payload files / bytes | 8 / 279 | 8 / 279 |
| Attachment payload files / bytes | 2 / 12,315 | 2 / 12,315 |
| EML files / bytes | 2 / 17,488 | 2 / 17,488 |
| Messages JSONL bytes | 23,086 | 23,765 |
| Extraction TAR bytes | 228,864 | 237,056 |
| Total extraction-output bytes | 273,908 | 282,103 |

The permanent fixture contract now locks each folder key, node, Unicode name, path, parent, item count and child count. It also locks the seven top-level message keys and subjects, their exact folder key/path/status, and the recovered child's separate root record. No ownership is inferred from search tables, hierarchy tables, folder counts, or discovery order.

## Tika Vertical 38 evidence

| Metric | Vertical 37 baseline | Vertical 38 result |
|---|---:|---:|
| Messages | 8 | 8 |
| Body records | 10 | 10 |
| Valid body payload files / bytes | 8 / 279 | 6 / 271 |
| Explicit unresolved HTML forms | 0 | 2 |
| Attachment payload files / bytes | 2 / 12,315 | 2 / 12,315 |
| EML files / bytes | 2 / 17,488 | 2 / 17,488 |
| Messages JSONL bytes | 23,765 | 23,865 |
| Bodies JSONL bytes | 2,820 | 2,922 |
| Extraction TAR bytes | 237,056 | 234,496 |
| Total extraction-output bytes | 282,103 | 279,543 |

The removed eight bytes were two Property Context HNID cells, not HTML. Parent and embedded-child plain bodies remain exact, both HTML forms are explicit unavailable records, and the DOCX, method-`5` payload, ownership and EML contracts remain unchanged.

## Latest completed work

Vertical 39 decodes ANSI version-14 and version-15 header diagnostics with ANSI-specific field widths and offsets while deliberately withholding all ANSI traversal roots. Exact behavior is protected by controlled synthetic tests, and all approved Unicode fixture contracts remain unchanged.

## Next evidence-based milestone

Obtain or reproducibly generate one approved, redistributable ANSI PST fixture, lock its provenance, size and SHA-256, and establish exact fail-closed header and root-page evidence before enabling any ANSI BBT/NBT traversal.

## Validation expectations

Every extraction PR must pass formatting, clippy with warnings denied, all Rust tests, CLI checks, Python wrapper checks, Docker build, approved fixture workflows, and exact artifact review. Unsupported or ambiguous candidates must remain unavailable rather than producing partial records.

## Risk statement

The current result is material evidence for approved Unicode fixtures plus synthetic ANSI header-layout tests, not broad PST compatibility. The sender's Exchange distinguished name is preserved but not SMTP-resolved. Real ANSI traversal, uncommon or corrupt layouts, nested embedded attachments, contacts/distribution lists, and many MAPI property combinations remain incomplete.
