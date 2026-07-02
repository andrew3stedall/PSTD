# PSTD v1 M16 Implementation Plan

## Implementation intent

M16 turns M15 triage into a concrete but bounded decoder expansion. It deliberately avoids broad parser changes and adds only one synthetic fixture-backed compact attachment-table path.

## Current foundation

- M13 records attachment table parse offsets, reasons, and table statuses.
- M14 classifies subnode layouts and performs bounded recursive loading.
- M15 produces compatibility triage categories and follow-up cases.

## Implemented M16 slice

1. Compact attachment-table decoder:
   - Recognises `CATB` synthetic fixture blocks.
   - Decodes row count and per-row filename/content-type/data lengths.
   - Builds normal attachment payload records using existing attachment helpers.
   - Preserves missing-payload status when rows have no bytes.
2. Compatibility triage expansion:
   - Adds fixture-backed decoder counts.
   - Adds compact attachment table as a supported compatibility case.
   - Adds serializable `CompatibilityTriageRecord` for archive output.
3. Extraction output:
   - Adds `data/compatibility_triage.jsonl`.
   - Adds manifest row for compatibility triage output.
   - Adds M16 status counters for fixture-backed decoder hits, triage records, and follow-up cases.

## Compact attachment-table shape

The M16 synthetic compact block shape is intentionally small and test-focused:

```text
CATB magic: 4 bytes
row count: u16
reserved: u16
repeated row:
  filename length: u16
  content type length: u16
  payload length: u32
  filename bytes: UTF-8
  content type bytes: UTF-8
  payload bytes
```

This is not claimed to represent every PST layout. It is a regression-backed decoder path used to keep expansion safe and measurable.

## Remaining work

- Add additional decoders only after a compatibility triage case and focused test exist.
- Add real fixture coverage only where source, licensing, and privacy handling are acceptable.
- Promote compatibility triage into external reporting if downstream tools need it outside the TAR archive.

## Definition of done

- M16 branch keeps CI green.
- Compact decoder success and missing-payload fallback are tested.
- Compatibility triage JSONL is emitted in the archive.
- Docs explain the decoder boundary and remaining limitations.
