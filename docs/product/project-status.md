# PSTD Project Status

_Last reviewed: 22 July 2026._

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
| ANSI Stage A fixture and traversal | Active / issue #475 | Build a deterministic Linux-generated version-14 PST with valid empty NBT and BBT leaves, independently validate it, and admit only bounded root-page decoding. It must emit zero objects and zero EML and does not establish ANSI email compatibility. |
| External PST implementations | Comparison-only tooling | External parsers and converters may be used offline or in an explicitly isolated fixture-generation/comparison workflow to create controlled fixtures and independently measure expected counts, properties, ownership, payload bytes, and MIME structure. They must not be required by the PSTD Rust library, CLI, Python wrapper, Docker image, or normal end-user runtime. PSTD acceptance still requires exact output from its own Rust implementation. |
| Downstream systems | Parked | Snowflake, UI, search, analytics, semantic search, and graph work remain out of scope. |

## Exact Tika baseline

| Metric | Current result |
|---|---:|
| Folders | 8 exact |
| Messages | 8, including one embedded child |
| Body records | 10 |
| Valid body payload files / bytes | 6 / 271 |
| Explicit unresolved HTML forms | 2 |
| Recipient records | 9 |
| Attachment records | 2 |
| Attachment payload files / bytes | 2 / 12,315 |
| EML files / bytes | 2 / 17,488 |
| Messages JSONL bytes | 23,865 |
| Bodies JSONL bytes | 2,922 |
| Recipients JSONL bytes | 2,708 |
| Attachments JSONL bytes | 1,240 |
| Extraction TAR bytes | 234,496 |

The method-`5` record `att_a9c94a13d70f1cb3` publishes a 453-byte `message/rfc822` payload with SHA-256 `86ffe5567da7aa505b8be16400889170ca583fd247cc0758f00a43c2a8a99420`. Those bytes are identical to standalone child `msg_0ff529af59d373d5.eml`. The parent EML remains exactly 17,035 bytes and includes only the validated method-`1` DOCX payload.

The eight removed body bytes from Vertical 38 were two Property Context HNID cells, not HTML. Parent and embedded-child plain bodies remain exact, and both HTML forms are explicit unavailable records.

## Latest completed work

The java-libpst comparison fixture now has a deterministic fail-closed baseline: 25 folders, 9 message metadata records, 12 body records, 0 recipients, 22 attachment metadata records, 0 materialised attachment payloads, 0 validated `IPM.Note*` classes, and 0 EML files. It adds corpus evidence but does not unlock an email extraction path.

## Next evidence-based milestone

Implement Stage A of issue #475: generate and validate a deterministic version-14 ANSI PST containing one empty NBT leaf and one empty BBT leaf, then admit only bounded ANSI root-page decoding.

Acceptance requires byte-identical regeneration, pinned length and SHA-256, independent byte-level validation, comparison with a pinned external reader, exact zero-object/zero-EML output, and no Unicode fixture regression. Stage A must not be described as ANSI email support.

After Stage A, the next ANSI milestone is one controlled folder and one plain-text `IPM.Note` message with exact recipient, body, EML path, byte length, and SHA-256. Broader Unicode attachment layouts, inline Content-ID handling, Exchange-to-SMTP mapping, and nested embedded messages remain queued after this explicitly ordered ANSI lane.

External implementations may be used to generate controlled PST fixtures and as independent comparison oracles. Any committed fixture must still have documented provenance or a reproducible generation recipe, redistribution permission, immutable bytes, byte length, SHA-256, and an exact expected contract. External tools remain outside PSTD's shipped dependency and runtime boundary.

## Validation expectations

Every extraction PR must pass formatting, clippy with warnings denied, all Rust tests, CLI checks, Python wrapper checks, Docker build, approved fixture workflows, and exact artifact review. Unsupported or ambiguous candidates must remain unavailable rather than producing partial records.

Comparison workflows must identify the external implementation and pinned version used, retain its raw report as evidence where licensing permits, and separately verify PSTD's own deterministic outputs. Agreement with another parser is supporting evidence, not sufficient proof when the format specification or fixture bytes contradict it.

## Risk statement

The current result is material evidence for two approved Unicode fixture paths plus synthetic ANSI header-layout tests, not broad PST compatibility. The sender's Exchange distinguished name is preserved but not SMTP-resolved. ANSI traversal and extraction remain unsupported until Stage A is independently validated and merged. Additional Unicode producers, uncommon or corrupt layouts, inline attachments, nested embedded attachments, contacts/distribution lists, appointments, and many MAPI property combinations remain incomplete.
