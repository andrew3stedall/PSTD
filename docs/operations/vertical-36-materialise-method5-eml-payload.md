# Vertical 36: Materialise method-5 child EML payload

_Last reviewed: 18 July 2026._

## Objective

Publish the already validated recovered-child EML as the exact payload of its authoritative method-`5` attachment record without changing the parent DOCX or standalone EML outputs.

## Before

- The method-`5` attachment linked to `msg_0ff529af59d373d5` through `embedded_message_key`.
- The child independently emitted a deterministic 453-byte plain-text EML.
- The attachment record remained metadata-only and its archive path had no file.
- Tika attachment payload output was 1 file / 11,862 bytes.

## Implementation

A shared `pstd::eml` module now owns deterministic plain-text EML construction. Extraction invokes the same builder after embedded messages are recovered and materialises a method-`5` payload only when all evidence is unique and valid:

- one attachment key and one non-empty child key;
- one child message and one directly owned text body;
- validated sender, recipient, subject and Date evidence;
- valid UTF-8 and safe single-line headers;
- no duplicate child links or nested child attachment ownership.

Successful records are updated to `message/rfc822`, retain their existing key, ordinal, owner, filename, archive path and `embedded_message_key`, and publish the exact EML bytes through `AttachmentPayload` and the manifest.

The standalone parent EML continues to include only supported by-value method-`1` attachments. Materialising method `5` therefore does not silently add the child to the parent's MIME tree.

## Exact Tika result

```text
parent message:                  msg_c6163b9157944cc9
method-5 attachment key:        att_a9c94a13d70f1cb3
child message:                   msg_0ff529af59d373d5
content type:                    message/rfc822
payload path:                    attachments/msg_c6163b9157944cc9/att_a9c94a13d70f1cb3_attachment_1.eml
payload bytes:                   453
payload SHA-256:                 86ffe5567da7aa505b8be16400889170ca583fd247cc0758f00a43c2a8a99420
standalone child byte identity:  exact
attachment payload files/bytes: 2 / 12,315
parent EML bytes:                17,035
child EML bytes:                 453
EML files/bytes:                 2 / 17,488
extraction TAR bytes:            228,864
total extraction-output bytes:  273,908
```

The DOCX remains 11,862 bytes with SHA-256 `0c87a742c970907d3b08c73e7834768abadd00fe4f4995a7dd98a206d4c494c0`. The parent EML remains 17,035 bytes. The child payload is byte-identical to the standalone child EML and no zero-byte placeholder exists.

## Fail-closed coverage

Focused integration tests prove that materialisation is rejected for:

- absent or mismatched child links;
- duplicate child links;
- nested child attachment ownership;
- more than one directly owned text body;
- unsafe header injection;
- candidates for which shared EML construction fails.

Unsupported candidates retain metadata-only status rather than selecting the first plausible message or producing partial bytes.

## Out of scope

- inserting the child as a MIME part in the outer parent EML;
- recursive or nested method-`5` materialisation;
- method-`5` layouts without authoritative recovered-child evidence;
- broad plain/HTML/RTF body selection changes.

## Next boundary

Lock complete folder and message coverage for `tika-testPST.pst`, including exact folder paths, message ownership, Unicode names, and preserved legacy Exchange address evidence before moving to the independent body-type fixture.
