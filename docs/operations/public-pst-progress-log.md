# Public PST Progress Log

_Last reviewed: 19 July 2026._

## Purpose

Track end-to-end conversion progress against the checked-in public PST fixture. This is the primary real-file quality signal and is separate from unit, lint, Docker, and CLI smoke checks.

## Mandatory milestone rule

After every extraction milestone:

1. inspect the `public-pst-progress` and milestone-specific artifacts;
2. record the extraction and diagnostic delta;
3. distinguish material fidelity progress from structural, diagnostic, or parser-only progress;
4. revise the next milestone from measured evidence;
5. avoid general compatibility claims based on one fixture;
6. keep artifacts bounded and exclude private PST data, complete message bodies, attachment payloads, and unredacted archives.

## Stable fixture baseline

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

The original fixture publishes these four complete row-aligned records through the production extraction path.

## Tika recipient, attachment, embedded-message, and child-EML evidence

PR #464 retains the eight-message and two-EML baseline, assigns all seven top-level messages through exact physical contents-table rows, and preserves the exact child EML as the linked method-`5` attachment payload.

| Evidence | Result |
|---|---:|
| Messages / body records | 8 / 10 |
| Body payload files / bytes | 8 / 279 |
| Recipient records | 9 |
| SMTP / raw-native rows | 6 / 3 |
| Attachment records | 2 |
| Attachment payload files / bytes | 2 / 12,315 |
| EML files / bytes | 2 / 17,488 |
| Recipient JSONL bytes | 2,708 |
| Top-level messages with exact physical owner | 7 |
| Messages JSONL bytes | 23,765 |
| Extraction TAR bytes | 237,056 |
| Total output bytes | 282,103 |

All seven top-level messages now resolve exactly to `/Début du fichier de données Outlook` through `node_802e` contents-table row keys. The recovered child owns one raw/native recipient, a 23-byte text body, four raw HTML-property bytes, and one exact 453-byte single-part plain-text EML. The method-`5` record publishes those exact bytes as `message/rfc822` at its stable archive path; the existing DOCX and unchanged 17,035-byte parent EML remain separate and byte-identical. No child value is attributed to the parent.

## Progress history

| Date | Milestone / PR range | Change type | Result | Next measured boundary |
|---|---|---|---|---|
| 2026-07-19 | Vertical 37 / #464 | Material folder ownership | Corrected table NID classification, decoded seven exact physical contents-table rows, locked all eight folder records and seven top-level owners, and preserved the separately linked embedded child and all payload contracts. | Validate independent body-form selection. |
| 2026-07-18 | Vertical 36 / #461 | Material embedded-message payload | Published the exact 453-byte child EML as `message/rfc822`, locked path/hash/ownership and byte identity, rejected ambiguous and nested candidates, and preserved parent EML/DOCX bytes. | Lock complete Tika folder and message coverage. |
| 2026-07-18 | Vertical 35 / #457 | Material child EML assembly | Emitted one exact 453-byte single-part plain-text child EML, gated admission through attachment metadata, preserved fail-closed top-level behaviour, and retained the parent/DOCX bytes. | Materialise the exact child EML as the method-`5` `message/rfc822` attachment payload. |
| 2026-07-17 | Vertical 34 / #455 | Material embedded-message extraction | Parsed the exact PtypObject wrapper, resolved one unique normal-message NID, emitted a separately keyed child with one recipient and two body records, linked it from method `5`, and preserved the parent/DOCX/EML contract. | Emit a plain-text-only child EML without promoting four invalid HTML bytes. |
| 2026-07-17 | Vertical 33 / #454 | Material EML assembly | Emitted one deterministic 17,035-byte Tika `multipart/mixed` EML with delivery-time-derived Date, valid plain text, one raw/native recipient, and the exact 11,862-byte DOCX payload; excluded unusable HTML and embedded-message method `5`. | Recover the method-`5` embedded message as a separate object. |
| 2026-07-16 | Vertical 32 / #452 | Material recipient extraction | Resolved heap-backed Table Context row matrices, attributed only direct recipient tables to each message, emitted eight records across seven Tika messages, preserved one legacy Exchange DN, and retained the exact DOCX payload. | Validate Date/required headers and assemble the first Tika `multipart/mixed` EML. |
| 2026-07-14 | Vertical 13 / #429 | New extraction representation | Added fail-closed complete recipient records retaining role, display name, address, and authoritative address kind by row. | Project names and addresses from the same rows and heap in one invocation. |
| 2026-07-14 | Verticals 11-12 / #427-#428 | Material recipient fidelity | Preferred `PidTagSmtpAddress`, then `PidTagEmailAddress`, over display-name fallback and classified the selected value as SMTP, native email address, or display name. The fixture produced four native email-address values. | Retain display names and addresses together rather than allowing address selection to replace names. |
| 2026-07-14 | Verticals 9-10 / #425-#426 | Structured recipient output | Assembled four role/name records and exposed bounded publication: two To and two Cc recipients with names `Recipient 1` through `Recipient 4`. | Extract address-bearing properties and preserve row alignment. |
| 2026-07-13 to 2026-07-14 | Verticals 4-8 / #420-#424 | End-to-end recipient string extraction | Extracted recipient string HNIDs, resolved Heap-on-Node values, decoded `PT_UNICODE`/`PT_STRING8`, built a fail-closed projection, and integrated recipient identity diagnostics into production reporting. | Convert validated identity values into row-aligned recipient records. |
| 2026-07-13 | Verticals 1-3 / #417-#419 | First semantic table values | Excluded internal LTP row properties, selected a supported non-internal fixed-width property, identified `PidTagRecipientType`, and published `to:to:cc:cc`. | Resolve recipient names and addresses from variable-width properties. |
| 2026-07-13 | PQ71-PQ74 / #413-#416 | Value decoding and production integration | Decoded supported fixed-width MAPI scalars, composed the validated projection, created bounded diagnostics, and attached them to production Table Context reporting. | Select the first real non-internal property and assign semantics only when authoritative. |
| 2026-07-12 to 2026-07-13 | PQ58-PQ70 / #400-#412 | Descriptor and row transport validation | Proved descriptor-to-bitmap mapping, published descriptor evidence, selected bounded row values, derived absolute offsets, built validated transport, and resolved row payload candidates. | Decode supported property types without relaxing bounds. |
| 2026-07-12 | PQ57 / #399 | Structural completion; output-neutral | Validated four 52-byte ordinal rows and exposed one exact 14-bit mask per row: `11111011000000`, with seven set and seven unset bits. | Prove descriptor mapping before accessing values. |
| 2026-07-11 to 2026-07-12 | PQ38-PQ56 / #379-#398 | Table Context and row-layout validation | Resolved the real TC heap, row-index BTH, subnode row storage, ordinal row references, 52-byte width, and bitmap boundary. | Preserve exact masks and map them safely. |
| 2026-07-10 | PQ37 / #378 | Parser primitive; output-neutral | Added specification-aligned TCINFO and TCOLDESC parsing while retaining unresolved typed HNID references. | Connect parsing to the real heap allocation. |
| 2026-07-10 | PQ36 / #377 | Major material extraction progress | Selected properties increased from 0 to 16; unknown properties decreased from 74 to 19; text and RTF bodies were recovered; fallback body rows decreased from 1 to 0; false table declarations were rejected. | Resolve the actual Table Context path rather than assuming the 208-byte payload shape. |
| 2026-07-10 | PQ32-PQ35 / #373-#376 | Assumption correction | Invalidated a false legacy descriptor interpretation, captured the raw boundary, identified a Unicode SLBLOCK, and resolved SLENTRY data/subnode targets with guards. | Decode permitted block encryption and tighten payload admission. |
| 2026-07-06 to 2026-07-10 | PQ4-PQ31 | Discovery and diagnostics | Corrected folder/message classification and developed the evidence needed to identify the real property, payload, subnode, and table boundaries. | Replace plausible-looking interpretations with specification-backed paths. |
| 2026-07-05 | PQ1-PQ3 / #197-#200 | Comparable baseline | Corrected root and B-tree traversal and established 50 BBT entries and 63 NBT entries as the stable structural baseline. | Improve folder and message fidelity. |

## Active boundary

Vertical 37 completes exact Tika folder and top-level message ownership validation. The next slice is independent plain-text, HTML and RTF body-form selection on `tika-various-body-types.pst`.

## Interpretation

The parser has advanced from structural discovery to material recipient, body, by-value attachment, parent/child EML, and exact method-`5` payload output on approved fixtures. This is still not broad compatibility: the Tika sender remains a raw native Exchange DN rather than resolved SMTP, nested embedded attachments and additional method-`5` layouts remain incomplete, and one fixture cannot establish support for uncommon or corrupt layouts.

## Completion report template

```text
Public PST progress:
- Fixture: tests/fixtures/pst/sample.pst
- BBT/NBT: 50/63
- Folders/messages/attachments: 11/1/0
- Selected/unknown properties: 16/19
- Body payloads: 2
- Recipient records: <status and bounded summary>
- Change vs previous milestone: <material progress|structural correction|diagnostic only|regression>
- Next blocker: <evidence-based statement>
```
