# Vertical 40 approved-fixture gap

_Last reviewed: 21 July 2026._

## Finding

The merged approved corpus does not currently prove a second by-value attachment method, format, or storage layout.

The exact Tika contract contains two attachment records:

- one method-`1` by-value DOCX payload of 11,862 bytes;
- one method-`5` embedded-message payload of 453 bytes, byte-identical to the separately emitted child EML.

The original public fixture contains no attachment payload. Existing code already records `PR_ATTACH_CONTENT_ID`, `PR_ATTACHMENT_HIDDEN`, MIME type, filename, method, declared size, and inline status, but no approved fixture contract currently demonstrates an inline/CID attachment or a second by-value layout.

## Consequence

No parser or MIME change is justified from the current fixtures. Implementing broader attachment handling now would be abstraction without new observable fixture behaviour.

## Required next evidence

Qualify one public, redistributable, immutable Unicode PST containing at least one normal email with one of:

1. a by-value attachment using a storage/data-tree layout not covered by the current DOCX;
2. an inline attachment with exact `PR_ATTACH_CONTENT_ID` and matching HTML `cid:` reference;
3. multiple by-value attachments whose ownership and ordinals can be locked exactly.

Admission must record immutable upstream revision, path, redistribution basis, byte length, SHA-256, NDB version, crypt method, exact folder/message/recipient/body/attachment counts, deterministic output, and every unsupported or ambiguous object.

The originating project must not become a PSTD build, runtime, test-runtime, CI, Docker, parser, or converter dependency.

## Fail-closed boundary

Do not infer inline status from filenames or MIME types, do not synthesize Content-ID values, and do not relax attachment ownership or payload validation merely to admit a candidate fixture. Unsupported, duplicate, out-of-range, ambiguous, or unowned attachment evidence remains explicit and emits no guessed MIME part.
