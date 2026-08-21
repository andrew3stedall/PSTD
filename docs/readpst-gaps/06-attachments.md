# Attachment parity

## RP-M1-04 provenance boundary

Attachment payloads in the canonical archive are now linked to their owning message
and archive path in data/evidence.jsonl, with size, SHA-256, bounded raw bytes, and
an explicit extraction status. This preserves the source trail needed by later
attachment parity work without changing the existing attachment output contract.

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

The `-a` option filters separate attachment files by extension. It must not destroy canonical metadata. PSTD’s equivalent now applies to separate files and generated mailbox/MSG MIME projections, and records:

```text
payload_status = extracted | filtered_by_extension | unavailable | unsupported | failed
filter_policy  = none | allow_list(<normalized extensions>)
```

The filter is case-insensitive, applies to the selected long/short filename according to the same documented rule, and leaves message attachment counts explainable. The current output slice is Partial: canonical attachment records remain unchanged, filtered records produce explicit decisions, and only allowed payloads enter separate files or generated MIME parts. Broad input and readpst differential coverage remains required.

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

## Post-RP-M7 output delivery

The attachment projection now shares one normalized extension predicate across mbox,
recursive mbox, MH, EML, separate files, KMail, Thunderbird mbox, and the MSG
compatibility EML/OLE path. Filtered attachments remain visible in adapter decisions and
canonical JSONL/payload evidence. MIME filenames additionally use the shared RFC 2231
encoder, retaining the original canonical filename separately from any ASCII fallback.

## Planned implementation — `RP-06`

### Readpst logic reviewed

`acceptable_ext` chooses `filename2` then `filename1`, accepts an absent filename or extension, and compares the extension case-insensitively against the NUL-separated `-a` list. `write_separate_attachment` chooses long/short/generated names, adds collision suffixes, resolves an ID-backed payload with `pst_getID`/`pst_attach_to_file`, and writes raw bytes. `write_inline_attachment` selects the MIME tag or `application/octet-stream`, emits base64 and Content-ID, applies RFC 2231 filename parameters, and reads memory or ID-backed data. `write_embedded_message` uses `attach->id2_head` to parse a child. `pst_parse_item` performs a second pass over attachment rows to connect ID2 values, child objects, and final data blocks. `msg.cpp` writes non-embedded attachments into OLE attachment storages with method-by-value metadata; embedded attachments are explicitly not implemented there.

### Planned PSTD model

Turn the current `AttachmentRecord`, `AttachmentMetadata`, `AttachmentPayload`, `attachment_table`, `attachment_property_context`, and `data_tree` paths into an `AttachmentEvidence` plus resolver:

```text
AttachmentEvidence {
  key, parent_item, ordinal, sequence, rendering_position,
  method, hidden, filename_original, filename_safe,
  mime_tag, content_id, declared_size,
  reference: Option<ReferenceEvidence>,
  payload: PayloadStatus + hash + archive_ref,
  embedded_item: Option<ItemKey>,
}

AttachmentResolver::resolve(method, property_context, id2, limits)
  -> ResolvedPayload | ScopedStatus
```

Implement method-specific resolvers for none/unknown, by-value, by-reference, by-reference-resolve, by-reference-only, embedded, and OLE. Keep a generic lossless byte path for OLE; do not make OLE decoding a prerequisite for extraction. Add an attachment graph index so `RP-07` can link embedded children without reparsing.

### Implementation flow

1. Parse attachment rows and property contexts into stable ordinals. Preserve row alignment, method, sequence, rendering position, flags, name candidates, MIME tag, CID, declared size, and source IDs.
2. Resolve long then short filename for compatibility, but retain every candidate and the reason for the selected name. Apply platform-independent sanitization only to output paths.
3. Resolve payloads by method through bounded data-tree/reference plans. Verify declared size when possible and hash bytes as they stream into canonical storage.
4. For by-reference methods, retain the reference node/block chain and distinguish absent, unresolved, ambiguous, corrupt, and empty payloads.
5. For embedded method 5, emit an edge to `RP-07`; do not inline child bytes until child classification and cycle checks succeed.
6. For OLE method 6, preserve exact bytes and metadata. A typed OLE projection can be added later without changing the raw contract.
7. Apply `-a` only in output projection. Normalize the extension once, record `filtered_by_extension`, and preserve the attachment count and metadata.
8. Correlate CIDs with HTML references using exact normalized IDs and unique matches. Record unmatched/duplicate matches rather than selecting heuristically.
9. Publish attachment files atomically and record path/hash/status in the manifest. Keep source order independent of filesystem enumeration.

### Improvements over readpst

- Never treat an unresolved reference as a zero-byte attachment or silently skip it.
- Preserve all name candidates and source identity; use stronger collision/path safety than `check_filename` and `f_name-name-N`.
- Use streaming hashes and size limits instead of loading every payload into memory.
- Detect duplicate attachment rows, ambiguous ID2 targets, cycles, and child ownership conflicts.
- Retain filtered payload metadata in canonical output; readpst’s `-a` only affects separate files.
- Keep Content-ID, hidden, disposition, sequence, and rendering position as independent facts; do not conflate inline semantics.

### Issue-ready acceptance

`RP-06A` covers row/property projection, `RP-06B` payload methods, `RP-06C` filename/path/filter policy, `RP-06D` CID/order, and `RP-06E` OLE/lossless evidence. Every issue needs positive and negative fixtures for direct, split, subnode, reference, embedded, OLE, zero-length, size-mismatch, duplicate-name, unsafe-name, non-ASCII, and unresolved cases. Verify payload hashes, source order, MIME projection, filtered statuses, and parent ownership; update [bodies](05-body-mime-and-rtf.md), [special items](07-embedded-and-special-email-items.md), [storage](09-storage-and-interoperability.md), and the matrix.

## RP-M3-01 graph consumption

Method-5 attachment rows now also feed the bounded `data/embedded_graph.jsonl`
projection. The graph retains source order, attachment ownership, child evidence,
observed bytes, and explicit missing/non-email/ambiguous/cycle/budget statuses. It
does not replace the attachment payload record or infer a child when the source ID2
reference is unavailable.
# RP-M2-03 delivery

The canonical `AttachmentRecord` now exposes a deterministic `source_ref` and
`rendering_position` alongside method, MIME type, CID, original/safe names, size
status, payload hash, archive path, and extraction status. By-value and embedded
message methods have explicit source labels; method-absent table rows remain
unresolved rather than being presented as a successful empty file. Existing Tika
attachment and embedded-message evidence verifies exact payload hashes, safe paths,
ordinals, and method/source correlation.

## RP-M2-04 MIME consumption

`data/mime_parts.jsonl` consumes attachment method/source, rendering position, safe
path, CID, payload hash, and embedded-child key from `AttachmentRecord`. Attachment
rows remain ordered by source rendering position; unresolved or size-mismatched rows
are represented as non-authoritative MIME parts rather than guessed zero-byte content.
