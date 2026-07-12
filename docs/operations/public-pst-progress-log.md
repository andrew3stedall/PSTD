# Public PST Progress Log

## Purpose

Track conversion progress against the checked-in public PST fixture after every parser-quality milestone. This is the primary real-file quality signal and is separate from unit, lint, Docker, and CLI smoke checks.

## Mandatory milestone rule

After each PQ pull request is green:

1. inspect the `public-pst-progress` and PQ-specific artifacts;
2. record the extraction and diagnostic delta;
3. distinguish fidelity progress from counter-only or parser-only progress;
4. revise the next PQ from measured evidence;
5. do not claim general PST compatibility from one fixture.

The artifact must contain summaries and bounded diagnostics only. It must not contain private PST files, complete message bodies, attachment payloads, or unredacted archives.

## Stable fixture baseline

| Metric | Current baseline |
|---|---:|
| BBT entries | 50 |
| NBT entries | 63 |
| Folder rows | 11 |
| True message candidates | 1 |
| Attachment rows | 0 |

## Recent progress

| Date | Milestone / PR | Change type | Public-fixture result | Next blocker |
|---|---|---|---|---|
| 2026-07-12 | PQ57 / #399 | Diagnostic completion; output-neutral | Validated four 52-byte ordinal rows and exposed one bounded 14-bit mask per row: `11111011000000` for all four. Each mask contains seven set and seven unset bits, excludes the two padding bits, and preserves 11 folders, 1/1 extracted message, 2 body payloads, 0 attachments, 16 selected properties, and 19 unknown properties. | Map each mask position through the TCINFO descriptor's `bitmap_bit` only after proving that the 14 indices are unique and complete. Report raw state, property tag, and property type without decoding row values or asserting semantic presence. |
| 2026-07-11 to 2026-07-12 | PQ38-PQ56 / #379-#398 | Table-context and row-layout validation | Resolved the real TC heap, row-index BTH, subnode-backed row payload, ordinal row references, 52-byte row width, TCINFO extents, and the bounded bitmap at bytes `50..52`. PQ56 measured seven set and seven unset bits per row without assigning property semantics. | Preserve the exact masks so column-index mapping can be validated before any value access. |
| 2026-07-10 | PQ37 / #378 | Parser primitive; output-neutral | Added bounded parsing for the 22-byte TCINFO root and exact 8-byte TCOLDESC records. Preserved `hidRowIndex`, `hnidRows`, and `hidIndex` as unresolved typed HNID values. Extraction counts and PQ36 property/body results are intentionally unchanged. | Wire parsing to the actual `hidUserRoot` heap allocation and emit fixture evidence before row materialisation. |
| 2026-07-10 | PQ36 / #377 | Material extraction progress | Decoded non-internal `NDB_CRYPT_PERMUTE` blocks, classified Heap-on-Node clients, and rejected invalid legacy table declarations. Selected properties increased **0 → 16**, unknown properties decreased **74 → 19**, text and RTF bodies were recovered, and fallback body rows decreased **1 → 0**. BID `0x74` remains an unresolved 208-byte payload. | Parse the real table-context root without assuming BID `0x74` is the row matrix. |
| 2026-07-10 | PQ35 / #376 | Structural correctness | Resolved Unicode SLENTRY targets through the BBT with depth and cycle guards. Corrected false table rows/values from 1/2 to 0/0 and exposed the remaining permissive payload-admission problem. | Decode payload encryption and require structural client admission. |
| 2026-07-10 | PQ34 / #375 | Diagnostic correction | Reinterpreted the 32-byte capture as a Unicode SLBLOCK with one 24-byte SLENTRY: `nid=0x692`, `bidData=0x7c`, `bidSub=0x7a`. Marked the previous zero-width table parse as spurious. | Resolve SLENTRY data and subnode targets. |
| 2026-07-10 | PQ33 / #374 | Evidence capture | Added bounded raw prefix capture for the selected table-like payload, allowing the SLBLOCK structure to be identified. | Interpret the captured payload boundary. |
| 2026-07-10 | PQ32 / #373 | Assumption invalidated | Demonstrated that one parsed row/two values with zero descriptor offsets and widths was not structurally credible. | Capture raw payload bytes rather than extending the legacy descriptor assumption. |
| 2026-07-09 | PQ27-PQ31 / #351-#370 | Diagnostic sequence | Propagated tag sources and descriptor fields, but evidence remained inconsistent with a valid table layout. | Validate descriptor and payload boundaries from raw evidence. |
| 2026-07-08 | PQ20-PQ26 / #337-#349 | Counter and interpretation sequence | Added row-matrix, parser, candidate, column, tag, and descriptor diagnostics. Extraction counts remained stable. | Identify the actual source/address space for table structures. |
| 2026-07-06 to 2026-07-08 | PQ11-PQ19 / #298-#334 | Source discovery | Progressed from missing heap-signature evidence to message subnode selection, decoding, classification, table probes, and membership/candidate measurement. | Follow the subnode/reference chain rather than guessing payload prefixes. |
| 2026-07-05 to 2026-07-06 | PQ4-PQ10 / #247-#292 | Fidelity and measurement | Corrected folder output to 11 rows and message output to one true candidate; exposed 74 unknown properties and no selected body property under the then-current path. | Repair real payload/Heap-on-Node traversal. |
| 2026-07-05 | PQ3 / #199 and #200 | Structured baseline | Corrected BBT/NBT traversal and established the comparable public-PST artifact: 50 BBT, 63 NBT, 1 root folder, 63 metadata candidates before later classification fixes. | Folder and message fidelity. |

## Interpretation

The headline result is not that PSTD is complete. PQ38-PQ57 establish a validated structural path from the real table heap to four bounded row bitmaps, but no row property values have been decoded and the bitmap states have not yet been mapped to TCINFO columns.

Do not treat parser-only milestones as extraction improvement. PQ58 must first prove that TCINFO `bitmap_bit` values form a unique, complete mapping for indices `0..13`; only later evidence may justify bounded value access.

## Completion report format

```text
Public PST progress:
- Fixture: tests/fixtures/pst/sample.pst
- BBT/NBT: 50/63
- Folders/messages/attachments: 11/1/0
- Selected/unknown properties: <values>
- Body outputs/fallbacks: <values>
- Change vs previous milestone: <material progress|structural correction|diagnostic only|regression>
- Next blocker: <evidence-based statement>
```
