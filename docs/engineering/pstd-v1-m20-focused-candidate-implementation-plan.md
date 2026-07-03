# PSTD v1 M20 Implementation Plan

## Implementation intent

M20 implements one focused candidate from the M19 selection output. The target is a high-priority attachment-table parse case that can be addressed without broad parser rewrites.

## Candidate selected

`unparseable_attachment_table`

## Implemented slice

1. Decoder detection:
   - Adds `CATW` compact attachment-table magic.
   - Keeps existing `CATB` support unchanged.
2. Row parsing:
   - Reuses the compact row structure used by `CATB`.
   - Interprets filename and content-type fields as UTF-16LE byte strings.
   - Treats payload bytes as raw attachment data.
3. Output:
   - Produces standard `AttachmentPayload` rows.
   - Uses existing safe filename and attachment metadata logic.
4. Fallback:
   - Odd UTF-16 byte lengths return parse errors.
   - Truncated headers or row data return parse errors.
   - Unknown table layouts still use existing table parser and fallback path.
5. Tests:
   - Successful `CATW` decode.
   - Invalid `CATW` row fallback.

## Data shape

```text
bytes 0..4   magic: CATW
bytes 4..6   row_count u16 LE
bytes 6..8   reserved u16 LE
per row:
  filename_len u16 LE
  content_type_len u16 LE
  data_len u32 LE
  filename UTF-16LE bytes
  content_type UTF-16LE bytes
  payload bytes
```

## Definition of done

- M20 branch keeps CI green.
- UTF-16 compact rows decode into attachment payloads.
- Malformed UTF-16 compact rows preserve parse-error fallback.
- Docs, issue plan, project status, and changelog are updated.
