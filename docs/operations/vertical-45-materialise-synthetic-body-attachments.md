# Vertical 45: materialise synthetic body attachments

_Last reviewed: 6 September 2026._

## Objective

Close ATT-10 by making standalone `pstd-eml` deliver every validated RTF and available
opaque encrypted body payload as an attachment-like artifact. The source body remains
the authoritative evidence; the attachment projection is explicitly synthetic and
non-authoritative.

## Implementation boundary

The adapter consumes the existing `BodyRecord` and `BodyPayload` collections. It selects
only `rtf`, `encrypted`, and `encrypted_html` body types, de-duplicates by stable body key,
and allocates synthetic attachment ordinals after ordinary attachment rows. RTF is
validated with the existing bounded direct/compressed validator before its original bytes
are copied. Encrypted and encrypted-HTML bytes remain opaque and are never decrypted,
decoded as HTML, or used to infer cleartext.

Each synthetic `AttachmentRecord` retains:

- the exact source bytes when available, with its payload hash and byte length;
- `source_body_key` and a deterministic `source_ref`;
- a safe deterministic filename (`.rtf` or `.encrypted*.bin`);
- `synthetic=true` and `authoritative=false`;
- explicit available, invalid, or source-unavailable extraction status.

Reference-shaped encrypted body values are rejected by the body loader and now survive as
explicit unavailable body records, allowing the EML adapter to report the missing source
without fabricating an empty attachment.

## Output contract

Inline mode emits exact source bytes as base64 MIME attachment parts alongside the normal
plain/HTML body tree. External mode emits the same bytes at validated relative attachment
paths and records the source/provenance/materialization state in `attachments.jsonl`.
Empty encrypted payloads are valid zero-byte artifacts. Invalid RTF and unavailable source
payloads remain manifest-visible metadata with no materialized file.

Ordinary attachment ordering, embedded-message handling, duplicate-key/path protection,
safe path validation, and payload hash/size integrity checks remain in force for both
ordinary and synthetic records.

## Validation

Focused adapter tests cover exact RTF and opaque encrypted bytes, empty encrypted payloads,
invalid RTF, unavailable sources, inline MIME/base64, external raw-file materialization,
manifest provenance, and integrity checks. The special-item fixture workflow additionally
runs the standalone adapter in inline and external modes twice, parses the generated MIME,
and compares decoded bytes, sizes, and SHA-256 values with the external manifest.

The canonical extraction contract remains source-backed: synthetic attachment projection
does not replace body records or claim decryption. Broad producer, method, and Purview
corpus coverage remains Partial elsewhere in the parity matrix.

## Next boundary

Admit a representative controlled Purview Unicode corpus and use it to measure the
remaining attachment Partial rows across producer layouts, references, OLE, CID, and
embedded-message combinations.
