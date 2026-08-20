# Attachment parity

## Attachment metadata

The libpst attachment model exposes more than a filename and payload. PSTD already has a useful `AttachmentRecord`, but parity requires the complete metadata and method behaviour below.

| Field/behaviour | readpst/libpst | PSTD status |
|---|---|---|
| Short filename (`PR_ATTACH_FILENAME`) | Fallback 8.3 filename. | **Partial** |
| Long filename (`PR_ATTACH_LONG_FILENAME`) | Preferred for output names. | **Partial**: preferred in current records; broad encoding not proven. |
| MIME tag | Used as Content-Type, with octet-stream fallback. | **Partial** |
| Content-ID | Emitted as `Content-ID` when present. | **Partial**: captured and emitted on validated paths; CID correlation is not proven. |
| Attachment method | 0 none, 1 by value, 2 by reference, 3 by-reference-resolve, 4 by-reference-only, 5 embedded message, 6 OLE. | **Partial**: method 1 and one method 5 layout are validated. |
| Rendering position | Indicates where an attachment appears in body text. | **Gap** |
| MIME sequence | Preserves MIME ordering. | **Gap** |
| Hidden flag | Can imply inline content. | **Partial** |
| Declared size | Used for diagnostics and payload checks. | **Partial** |
| Exact payload bytes | Reads direct, subnode, reference, and multi-block data. | **Partial**: one Unicode XBLOCK DOCX path is exact. |

## Attachment methods

### By value (`1`)

PSTD must support direct binary payloads, subnode-backed payloads, XBLOCK/XXBLOCK chains, large payloads, zero-length payloads, and declared-size mismatches. Exact byte length and SHA-256 belong in the canonical record.

### By reference (`2`, `3`, `4`)

The reference value must be resolved through the appropriate ID2 tree and source node. The result must retain:

```text
reference_value
resolved_source_node
resolved_block_chain
resolution_status
payload_status
```

An unresolved reference is not an empty attachment. It is metadata plus `unavailable_reference` or `failed_reference_resolution`, with bounded evidence.

### Embedded message (`5`)

The child message needs a separate stable message identity and a parent attachment link. A valid child can be emitted as `message/rfc822`; its own body, recipients, headers, attachments, and nested children must be processed under an explicit recursion limit. Ambiguous or non-email embedded objects must remain linked metadata with a scoped skip status.

PSTD currently materializes one method-5 child EML layout and deliberately defers broader nesting. This is Partial, not complete parity.

### OLE (`6`)

readpst treats non-embedded attachment data as a file/MIME payload when bytes or a resolvable ID exist. PSTD must preserve OLE bytes and metadata without attempting to reinterpret the object as a normal email attachment. A later typed OLE decoder is optional; lossless extraction is not.

## Inline and CID behaviour

readpst emits `Content-ID` but does not itself prove a full HTML `cid:` relationship model. PSTD should provide the stronger structured behaviour:

- preserve `content_id` exactly after safe normalization;
- retain `is_inline` from hidden/Content-ID evidence separately from disposition choice;
- expose HTML references and their matched attachment keys when correlation is unique;
- retain unmatched HTML CIDs and unmatched inline attachments as explicit diagnostics;
- never mark a CID as resolved merely because a filename looks similar.

## Attachment filtering

The `-a` option filters separate attachment files by extension. It must not destroy canonical metadata. PSTD’s equivalent should record:

```text
payload_status = extracted | filtered_by_extension | unavailable | unsupported | failed
filter_policy  = none | allow_list(<normalized extensions>)
```

The filter must be case-insensitive, apply to the selected long/short filename according to the same documented rule, and leave message attachment counts explainable.

## Filename and path safety

Implement deterministic, platform-independent sanitization for control characters, path separators, reserved names, trailing dots/spaces, empty names, and collisions. Keep `filename_original` separate from `filename_safe`. RFC 2231 parameter encoding belongs to MIME output; it must not mutate the canonical original filename.

## Attachment acceptance matrix

The fixture corpus must include, at minimum:

1. one by-value payload split across several blocks;
2. multiple attachments with ordering and duplicate names;
3. non-ASCII long and short names;
4. inline image with Content-ID and HTML reference;
5. unresolved by-reference and reference-only objects;
6. embedded message with its own attachment;
7. nested embedded message beyond the first level, bounded and deterministic;
8. OLE bytes;
9. zero-length and declared-size-mismatch payloads;
10. filtered extensions and unnamed attachments.
