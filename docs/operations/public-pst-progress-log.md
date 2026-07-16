# Public PST Progress Log

_Last reviewed: 16 July 2026._

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

## Tika recipient and attachment evidence

Draft PR #452 advances `tests/fixtures/upstream/tika-testPST.pst` from zero to eight recipient records across seven messages while preserving the existing DOCX result.

| Evidence | Result |
|---|---:|
| Messages / body records | 7 / 8 |
| Recipient records | 8 |
| SMTP / raw-native rows | 6 / 2 |
| Attachment records | 1 |
| Attachment payload files / bytes | 1 / 11,862 |
| Recipient JSONL bytes | 2,418 |
| Extraction TAR bytes | 202,752 |
| Total output bytes | 241,579 |

The raw/native rows preserve the complete legacy Exchange distinguished name for `Hong-Thai Nguyen` and the attachment owner's validated `PidTagEmailAddress`. Neither is relabelled as SMTP without a validated `PidTagSmtpAddress` projection.

## Progress history

| Date | Milestone / PR range | Change type | Result | Next measured boundary |
|---|---|---|---|---|
| 2026-07-16 | Vertical 32 / draft #452 | Material recipient extraction | Resolved heap-backed Table Context row matrices, attributed only direct recipient tables to each message, emitted eight records across seven Tika messages, preserved one legacy Exchange DN, and retained the exact DOCX payload. | Validate Date/required headers and assemble the first Tika `multipart/mixed` EML. |
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

## Active unmerged work

Draft PR #452 bridges heap-backed recipient rows into the existing validated projection, filters production ownership through each message root's direct recipient-table NIDs, and locks the eight-record Tika result in a permanent fixture gate. It is not part of the merged baseline until its exact final head passes CI and merges.

## Interpretation

The parser has advanced from structural discovery to material recipient, body, attachment, and readable-EML output on approved fixtures. This is still not broad compatibility: the Tika attachment message has no assembled EML yet, embedded messages remain deferred, and one fixture cannot establish support for uncommon or corrupt layouts.

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
