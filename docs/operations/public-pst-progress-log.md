# Public PST Progress Log

_Last reviewed: 22 July 2026._

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
| EML files / bytes | 2 / 17,488 |
| Messages JSONL bytes | 23,865 |
| Bodies JSONL bytes | 2,922 |
| Recipients JSONL bytes | 2,708 |
| Attachments JSONL bytes | 1,240 |
| Extraction TAR bytes | 234,496 |

All seven top-level messages resolve exactly to `/Début du fichier de données Outlook` through `node_802e` contents-table row keys. The recovered child owns one raw/native recipient, a 23-byte text body, and one explicit unavailable HTML form. It emits an exact 453-byte single-part plain-text EML. The method-`5` record publishes those same bytes as `message/rfc822` at its stable archive path. The existing DOCX and unchanged 17,035-byte parent EML remain separate and byte-identical. No child value is attributed to the parent.

The method-`5` record `att_a9c94a13d70f1cb3` publishes a 453-byte payload with SHA-256 `86ffe5567da7aa505b8be16400889170ca583fd247cc0758f00a43c2a8a99420`, byte-identical to `msg_0ff529af59d373d5.eml`.

## Progress history

| Date | Milestone / PR | Change type | Result | Next measured boundary |
|---|---|---|---|---|
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

Verticals 35-39 are complete and must not be duplicated. The next slice is broader Unicode email compatibility on approved evidence, preferring another by-value attachment method, format, or storage layout, followed by inline attachment and Content-ID handling.

Pinned external PST implementations may be used offline or in explicitly isolated fixture-generation and comparison workflows to create controlled fixtures and independently inventory counts, ownership, properties, payload bytes, hashes, and MIME structure. They must not become required PSTD runtime or normal validation dependencies, and PSTD acceptance must still come from its own deterministic Rust output.

## Interpretation

The parser has advanced from structural discovery to material recipient, body, by-value attachment, parent/child EML, exact method-`5` payload output, exact Tika folder ownership, independent body-form admission, and fail-closed ANSI header diagnostics. This is still not broad compatibility: the Tika sender remains a raw native Exchange DN rather than resolved SMTP, nested embedded attachments and additional method-`5` layouts remain incomplete, and two fixtures cannot establish support for uncommon or corrupt layouts.

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
