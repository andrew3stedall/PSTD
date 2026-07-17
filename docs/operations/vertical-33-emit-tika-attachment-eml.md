# Vertical 33: Emit the first Tika attachment EML

_Last reviewed: 17 July 2026._

## Objective

Assemble one deterministic attachment-bearing EML for `msg_c6163b9157944cc9` from the message, recipient, body, Date, and DOCX evidence already emitted by the production extractor.

## Evidence correction

The prior roadmap described the message as having usable plain text and HTML. The fixture proves otherwise:

- the plain-text payload is valid UTF-8, 22 bytes, and contains `Forwarding mail…\r\n\r\n`;
- the four-byte HTML payload is `bf 83 00 00`, is not valid UTF-8, and is not usable markup;
- `transport_message_headers` and submit time are absent;
- `PidTagMessageDeliveryTime` is present as FILETIME `132509026800000000`;
- one directly owned To recipient and one 11,862-byte by-value DOCX payload are already validated.

The safe result is therefore a plain-text `multipart/mixed` message, not a fabricated HTML alternative.

## Implementation

- Group attachment payloads by message key and stable attachment ordinal.
- Require Date evidence in this order: validated transport Date, submit-time FILETIME, then delivery-time FILETIME.
- Convert the bounded delivery FILETIME to `Thu, 26 Nov 2020 22:18:00 +0000`.
- Preserve the existing `multipart/alternative` path for non-attachment messages.
- Emit a plain-text body inside `multipart/mixed` when no validated HTML is available.
- Base64-wrap attachment bytes at 76 characters per CRLF line.
- Map `.docx` to `application/vnd.openxmlformats-officedocument.wordprocessingml.document`.
- Require every emitted attachment to be non-empty, method `1`, strictly ordered, length-matched, and SHA-256-matched.
- Keep method-`5` embedded-message payloads outside this vertical.

## Observable result

| Evidence | Result |
|---|---:|
| EML files | 1 |
| EML bytes | 17,035 |
| Plain-text MIME parts | 1 |
| HTML MIME parts | 0 |
| DOCX MIME parts | 1 |
| Decoded DOCX bytes | 11,862 |
| Decoded DOCX SHA-256 | `0c87a742c970907d3b08c73e7834768abadd00fe4f4995a7dd98a206d4c494c0` |
| Structured extraction delta | none |
| Extraction TAR / total bytes | 202,752 / 241,579 |

The emitted file is `msg_c6163b9157944cc9.eml`. Its DOCX MIME part decodes byte-for-byte to the extracted attachment and retains filename `attachment.docx`.

## Header and address boundary

The To row preserves display name `'lfcnassif@gmail.com'` and raw address `lfcnassif@gmail.com` without relabelling it as authoritative SMTP. The From header preserves Luis Filipe da Cruz Nassif's native Exchange distinguished name because no validated SMTP projection is available. This is forensic preservation, not SMTP resolution.

## Before versus after

| Measure | Before | After |
|---|---:|---:|
| Messages | 7 | 7 |
| Body records | 8 | 8 |
| Recipient records | 8 | 8 |
| Attachment records | 1 | 1 |
| Attachment payload files / bytes | 1 / 11,862 | 1 / 11,862 |
| EML files / bytes | 0 / 0 | 1 / 17,035 |
| Extraction TAR bytes | 202,752 | 202,752 |
| Total extraction-output bytes | 241,579 | 241,579 |

## Validation

Focused Rust tests cover text-only mixed assembly, FILETIME Date conversion, attachment grouping and ordering, boundary handling, invalid attachment methods, and missing Date rejection. The permanent Tika workflow verifies:

- exact filename and 17,035-byte size;
- CRLF-only line endings;
- parsed top-level `multipart/mixed`;
- exact Subject, To, Date, and preserved native sender evidence;
- absence of an HTML MIME part;
- exact decoded plain-text body;
- one DOCX part with registered MIME type;
- byte-identical decoded attachment payload.

The original readable EML and RTF/HTML fixture workflows remain unchanged and continue to protect the 956-byte `multipart/alternative` message.

## Next vertical milestone

Recover the method-`5` embedded message as a separate message object and attachment path. Retain the current direct-ownership boundary so its recipients, body, and identifiers cannot be projected onto the outer message.
