# Vertical 35: Emit the Tika embedded child as plain-text EML

_Last reviewed: 18 July 2026._

## Objective

Emit the separately recovered method-`5` child as a deterministic attachmentless `text/plain` EML without admitting unrelated plain-only messages or promoting the child's four raw `PidTagHtml` bytes to MIME HTML.

## Merged implementation

PR #457 adds an explicit plain-text-only EML policy to `pstd-eml`. The policy is authorised only for message keys referenced by an `AttachmentRecord.embedded_message_key`. The authoritative link comes from attachment metadata rather than `attachment_payloads`, because the method-`5` attachment does not yet have materialised bytes.

Ordinary attachmentless messages without validated HTML retain the previous fail-closed behaviour. This prevents plausible-looking output from three Tika top-level records whose three-byte body evidence currently decodes as isolated characters and has not been validated as usable message text.

## Exact Tika evidence

| Evidence | Result |
|---|---|
| Parent EML | `msg_c6163b9157944cc9.eml`, unchanged, 17,035 bytes |
| Child EML | `msg_0ff529af59d373d5.eml`, 453 bytes |
| Child SHA-256 | `86ffe5567da7aa505b8be16400889170ca583fd247cc0758f00a43c2a8a99420` |
| MIME shape | Single-part `text/plain; charset=utf-8`, `8bit` transfer encoding |
| Child body | Exact 23-byte source body with deterministic CRLF assembly |
| Required headers | From, To, Subject, Date, Message-ID, MIME-Version |
| Raw HTML evidence | `7f 83 00 00`, preserved in structured extraction and absent from EML |
| Tika EML count / bytes | 2 / 17,488 |

The fixture also proves that the child remains non-multipart, contains no HTML part or PSTD multipart boundary, preserves native Exchange sender and recipient evidence without inventing SMTP, and leaves the parent's DOCX payload and MIME bytes unchanged.

## Fail-closed boundary

Plain-text-only EML is not generally enabled for every UTF-8-decodable body. It is currently admitted only for a separately classified child referenced by authoritative attachment metadata and still requires validated sender, recipient, subject, Date evidence, and plain text. Missing, ambiguous, malformed, or unlinked candidates remain unavailable.

## Unchanged structured extraction

PR #457 changes EML assembly and fixture evidence only. The Tika structured baseline remains:

- 8 messages;
- 10 body records and 8 body payload files totalling 279 bytes;
- 9 recipient records;
- 2 attachment records;
- 1 by-value attachment payload totalling 11,862 bytes;
- 227,840-byte extraction TAR;
- 272,884 bytes of extraction output.

## Next measured boundary

Materialise the exact 453-byte child EML as the method-`5` attachment payload with:

- content type `message/rfc822`;
- deterministic attachment key and archive path;
- exact byte length and SHA-256;
- explicit parent-child ownership;
- no empty placeholder payload;
- no recursion or nested-attachment claims beyond the approved fixture.

Whether the outer parent EML should immediately include that payload as a MIME attachment must be validated explicitly rather than inferred from payload availability.
