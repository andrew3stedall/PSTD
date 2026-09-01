# Public PST Progress Log

_Last reviewed: 1 September 2026._

## Purpose

Track end-to-end conversion progress against the checked-in public PST fixtures. This is the primary real-file quality signal and is separate from unit, lint, Docker, and CLI smoke checks.

## Mandatory milestone rule

After every extraction milestone:

1. inspect the `public-pst-progress` and milestone-specific artifacts;
2. record the extraction and diagnostic delta;
3. distinguish material fidelity progress from structural, diagnostic, or parser-only progress;
4. revise the next milestone from measured evidence;
5. avoid general compatibility claims based on one fixture;
6. keep artifacts bounded and exclude private PST data, complete message bodies, attachment payloads, and unredacted archives.

## Stable original-fixture baseline

| Metric | Current result |
|---|---:|
| BBT entries | 50 |
| NBT entries | 63 |
| Folder rows | 11 |
| True message candidates | 1 |
| Extracted messages | 1 |
| Body payloads | 2 |
| Attachment rows emitted | 0 |
| Selected properties | 16 |
| Unknown properties | 19 |
| Validated Table Context rows | 4 × 52 bytes |

## Latest validated recipient evidence

| Row | Recipient role | Display name | Address property | Value | Classification |
|---:|---|---|---|---|---|
| 0 | To | Recipient 1 | `PidTagEmailAddress` | `to1@domain.com` | native email address |
| 1 | To | Recipient 2 | `PidTagEmailAddress` | `to2@domain.com` | native email address |
| 2 | Cc | Recipient 3 | `PidTagEmailAddress` | `cc1@domain.com` | native email address |
| 3 | Cc | Recipient 4 | `PidTagEmailAddress` | `cc2@domain.com` | native email address |

The original fixture publishes these four complete row-aligned records through the production extraction path and emits one deterministic 956-byte `multipart/alternative` EML.

## Exact Tika baseline through Vertical 38

| Evidence | Result |
|---|---:|
| Folders | 8 exact |
| Messages | 8, including one embedded child |
| Body records | 10 |
| Valid body payload files / bytes | 6 / 271 |
| Explicit unresolved HTML forms | 2 |
| Recipient records | 9 |
| Attachment records | 2 |
| Attachment payload files / bytes | 2 / 12,315 |
| EML files / bytes | 2 / deterministic; inline parent carries both recovered payloads |
| Messages JSONL bytes | 23,865 |
| Bodies JSONL bytes | 2,922 |
| Recipients JSONL bytes | 2,708 |
| Attachments JSONL bytes | 1,240 |
| Extraction TAR bytes | 234,496 |

All seven top-level messages resolve exactly to `/Début du fichier de données Outlook` through `node_802e` contents-table row keys. The recovered child owns one raw/native recipient, a 23-byte text body, and one explicit unavailable HTML form. It emits an exact 453-byte single-part plain-text EML. The method-`5` record publishes those same bytes as `message/rfc822` at its stable archive path. Inline parent EML now carries both the DOCX and recovered child payload; external mode publishes them at manifest-linked paths. No child value is attributed to the parent’s metadata records.

The method-`5` record `att_a9c94a13d70f1cb3` publishes a 453-byte payload with SHA-256 `86ffe5567da7aa505b8be16400889170ca583fd247cc0758f00a43c2a8a99420`, byte-identical to `msg_0ff529af59d373d5.eml`.

## Progress history

| Date | Milestone / PR | Change type | Result | Next measured boundary |
|---|---|---|---|---|
| 2026-08-31 | Vertical 42 / #573 | Per-context String8 code-page selection | Supported `PR_MESSAGE_CODEPAGE`/`PR_INTERNET_CPID` declarations now select UTF-8, Windows-1252, or ISO-8859-1 before String8 decoding across message, folder, attachment-table/property-context, and embedded paths; raw declarations and provenance are retained, while malformed, unsupported, and conflicting evidence falls back explicitly. No approved public-fixture metric changed because this is bounded charset-fidelity evidence. | Controlled PST with authoritative per-message String8 code-page metadata and measured non-ASCII body/header/attachment names. |
| 2026-08-31 | Vertical 43 / #575 | Expanded String8 code-page decoding | Added pinned `encoding_rs` decoding for Shift-JIS/932, GBK/936, EUC-KR/949, and Big5/950 across the existing property-context propagation paths; `-C` remains authoritative, raw bytes/provenance remain available, and malformed multibyte sequences publish explicit conversion-error counts. No approved public-fixture metric changed because this is bounded charset-fidelity evidence. | Controlled PST with producer-authored non-Western String8 properties and measured body/header/attachment-name output. |
| 2026-08-31 | Vertical 41 / #569 | Structural indirect OLE attachment evidence | Method-6 property-context references now resolve through both wide and compact SLBLOCK forms to exact arbitrary OLE/Compound File bytes; duplicate, missing, and malformed mappings fail closed. No public PST metric changed because this is bounded synthetic regression evidence. | Approved OLE-bearing PST or Purview Unicode export with a measured method-6 payload. |
| 2026-08-31 | Vertical 40 / #475 | Structural ANSI Stage-A fixture | Deterministic 2,048-byte v14 ANSI PST generated by Linux Rust, independently weak-CRC/page-trailer validated, accepted by libpff pffinfo 20180714, and classified by PSTD as fail-closed empty ANSI traversal (`partial`, extraction disabled); SHA-256 `b5de1ce4cebacc2ea4cefddb4ab9c4d32e5fed04b81cd681e8831faf1323c765`. | ANSI Stage B one-folder/one-message fixture or broader approved Unicode corpus. |
| 2026-09-01 | Vertical 44 / #577 | Material ANSI Stage-B message fixture | Deterministic Linux Rust v14 PST with one `Synthetic Mail` folder, one `IPM.Note` message, one structured To recipient, ANSI String8 subject/sender/body/raw transport headers, and no attachments; independent byte validation, repeat equality, PSTD canonical JSONL, and exact plain-text EML are required. External-reader acceptance/rejection is retained as explicit evidence rather than a broad compatibility claim. | ANSI attachment, HTML/RTF, typed-item, malformed-derivative, or producer-specific fixtures. |
| 2026-09-01 | Attachment vertical / #579 | ANSI by-value attachment fixture | Deterministic Linux Rust v14 PST with one folder, one message, and one ANSI/String8 property-context method-1 attachment (`ansi-attachment.bin`, `application/octet-stream`); exact payload hash `fc1107a00b29da722c39c00794f0458c1626402f8eeab7f080ce596ba01142c1`, independent validator, canonical bytes, inline MIME/base64, external raw-file/manifest, repeat, overwrite, and truncated fail-closed checks all pass. This is one controlled layout, not broad ANSI evidence. | ANSI reference/embedded/OLE, HTML/RTF, typed-item, malformed-derivative, producer-specific, and broader corpus fixtures. |
| 2026-09-01 | Attachment vertical / #580 | ANSI indirect by-value attachment fixture | Extended #579's deterministic ANSI v14 fixture with a method-1 `PR_ATTACH_DATA_OBJ` HNID `0x311` resolved through an exact root SLBLOCK mapping to a separate payload BID; direct and indirect variants share the independently validated payload hash and pass canonical, inline, external, repeat, overwrite, and truncation gates. This is one controlled indirect layout, not broad reference-method or producer evidence. | ANSI methods 2/3/4, embedded/OLE producer layouts, HTML/RTF, typed-item, malformed-derivative, and broader corpus fixtures. |
| 2026-09-01 | Attachment vertical / #581 | ANSI method-2 by-reference attachment fixture | Added a deterministic ANSI v14 method-2 variant preserving `PR_ATTACH_METHOD=2` while resolving HNID `0x311` through the root SLBLOCK to the exact payload BID; direct, indirect method-1, and method-2 variants pass independent structure, canonical hash, inline MIME/base64, external manifest/raw-file, repeat, overwrite, and truncation gates. This is one controlled reference mapping, not methods 3/4 or broad ID2/path producer evidence. | ANSI methods 3/4, external ID2/path layouts, embedded/OLE producer layouts, HTML/RTF, typed-item, malformed-derivative, and broader corpus fixtures. |
| 2026-09-01 | Attachment vertical / #582 | ANSI method-3 by-reference attachment fixture | Added a deterministic ANSI v14 method-3 variant preserving `PR_ATTACH_METHOD=3` while resolving HNID `0x311` through the root SLBLOCK to the exact payload BID; direct, indirect method-1, method-2, and method-3 variants pass independent structure, canonical hash, inline MIME/base64, external manifest/raw-file, repeat, overwrite, and truncation gates. This is one controlled reference mapping, not method 4, external ID2/path, or broad producer evidence. | ANSI method 4, external ID2/path layouts, embedded/OLE producer layouts, HTML/RTF, typed-item, malformed-derivative, and broader corpus fixtures. |
| 2026-07-20 | Vertical 39 / #473 | Diagnostic-only ANSI header support | Decoded version-14/15 root offsets and crypt-method locations with ANSI-specific widths while preventing those values from authorising traversal. | Broaden approved Unicode fixture coverage. |
| 2026-07-20 | Vertical 38 / #470 | Material body-form admission | Rejected four-byte Property Context body locators as unavailable, retained valid plain-text siblings, and preserved attachment and EML bytes. | Broaden attachment/layout evidence. |
| 2026-07-19 | Vertical 37 / #464 | Material folder ownership | Corrected table NID classification, decoded seven exact physical contents-table rows, locked all eight folder records and seven top-level owners, and preserved the separately linked embedded child and all payload contracts. | Validate independent body-form selection. |
| 2026-07-18 | Vertical 36 / #461 | Material embedded-message payload | Published the exact 453-byte child EML as `message/rfc822`, locked path/hash/ownership and byte identity, rejected ambiguous and nested candidates, and preserved parent EML/DOCX bytes. | Lock complete Tika folder and message coverage. |
| 2026-07-18 | Vertical 35 / #457 | Material child EML assembly | Emitted one exact 453-byte single-part plain-text child EML, gated admission through attachment metadata, preserved fail-closed top-level behaviour, and retained the parent/DOCX bytes. | Materialise the exact child EML as the method-`5` payload. |
| 2026-07-17 | Vertical 34 / #455 | Material embedded-message extraction | Parsed the PtypObject wrapper, resolved one unique normal-message NID, emitted a separately keyed child with one recipient and two body records, and preserved the parent output contract. | Emit a plain-text-only child EML. |
| 2026-07-17 | Vertical 33 / #454 | Material EML assembly | Emitted one deterministic 17,035-byte Tika `multipart/mixed` EML with Date, valid plain text, one raw/native recipient, and the exact 11,862-byte DOCX payload. | Recover the method-`5` embedded message. |
| 2026-07-16 | Vertical 32 / #452 | Material recipient extraction | Resolved heap-backed Table Context row matrices and emitted directly owned recipient records across Tika messages. | Assemble the first Tika attachment EML. |
| 2026-07-14 | Vertical 13 / #429 | New extraction representation | Added fail-closed complete recipient records retaining role, display name, address, and authoritative address kind by row. | Integrate readable EML. |
| 2026-07-13 to 2026-07-14 | Verticals 1-12 / #417-#428 | Material recipient fidelity | Progressed from recipient-role decoding to row-aligned names and address selection. | Publish complete records. |
| 2026-07-10 to 2026-07-13 | PQ36-PQ74 / #377-#416 | Parser and transport foundation | Recovered bodies, resolved the Table Context path and row transport, decoded supported values, and integrated bounded diagnostics. | Implement semantic extraction verticals. |
| 2026-07-05 to 2026-07-10 | PQ1-PQ35 | Structural discovery | Corrected root/index traversal and replaced false table assumptions with measured evidence. | Resolve the real Table Context path. |

## Active boundary

Verticals 35-44 and the ANSI by-value/reference attachment slices are complete when #582's workflow is green and merged; they must not be duplicated. ANSI Stage A remains a structural baseline, Stage B adds one controlled email/EML shape, and Stage C now covers direct and indirect method-1 plus method-2 and method-3 HNID/SLBLOCK attachment layouts. The next ANSI slices require separate fixtures for method 4, external ID2/path references, embedded/OLE attachments, HTML/RTF bodies, typed items, malformed derivatives, and producer-specific layouts. In parallel, an approved OLE-bearing PST or Purview Unicode export with measured attachment bytes remains the highest-value external corpus target.

Pinned external PST implementations may be used offline or in explicitly isolated fixture-generation and comparison workflows to create controlled fixtures and independently inventory counts, ownership, properties, payload bytes, hashes, and MIME structure. They must not become required PSTD runtime or normal validation dependencies, and PSTD acceptance must still come from its own deterministic Rust output.

## Interpretation

The parser has advanced from structural discovery to material recipient, body, by-value attachment, parent/child EML, exact method-`5` payload output, exact Tika folder ownership, independent body-form admission, a libpff-accepted ANSI structural baseline, and controlled ANSI email/EML plus direct/indirect method-1, method-2, and method-3 attachment paths. This is still not broad compatibility: the Tika sender remains a raw native Exchange DN rather than resolved SMTP, nested embedded attachments and additional method-`5` layouts remain incomplete, indirect OLE evidence is still bounded to synthetic property-context wide/compact references, ANSI method 4, external ID2/path references, embedded/OLE producer layouts, HTML/RTF, typed items, and producer breadth remain unproven, and the approved fixtures cannot establish support for uncommon or corrupt layouts.

## Completion report template

```text
Public PST progress:
- Fixture: <approved immutable fixture path>
- Fixture provenance/version/SHA-256: <exact values>
- Folders/messages/attachments: <exact counts>
- Body/recipient/attachment payloads: <exact counts and bytes>
- EML outputs: <exact paths, bytes, hashes, and MIME structure>
- Change vs previous milestone: <material progress|structural correction|diagnostic only|regression>
- Unsupported boundary: <explicit fail-closed cases>
- Next blocker: <evidence-based statement>
```
