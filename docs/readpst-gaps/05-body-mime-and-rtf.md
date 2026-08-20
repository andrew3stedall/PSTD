# Bodies, MIME, and RTF

## Body evidence

readpst can emit or use the following body sources:

| Source | Upstream use | PSTD status |
|---|---|---|
| `PR_BODY` / String8 equivalent | Plain-text body. | **Partial**: validated on approved fixtures; charset breadth is incomplete. |
| `PR_HTML` / `PR_HTML_STRING` | HTML body, including binary and string forms. | **Partial**: raw HTML and one RTF-derived HTML path exist; binary locator and producer breadth remain incomplete. |
| `PR_RTF_COMPRESSED` | LZFU-compressed RTF, optionally decompressed to `rtf-body.rtf` or used to recover HTML. | **Partial**: decompression and one `fromhtml` path are tested; generic RTF preservation and MIME policy are incomplete. |
| `PR_REPORT_TEXT` | Delivery-status/report body. | **Gap** |
| encrypted text/HTML bodies | Preserved as attachment-like bytes when present; not interpreted as cleartext. | **Gap** |

PSTD must retain independent body records when several representations exist. A failed HTML decode must not suppress a valid plain body, and a raw four-byte locator must remain unavailable rather than being emitted as HTML.

## MIME structures

The readpst email writer produces these observable structures:

```text
ordinary email with text only       -> multipart/mixed containing text/plain
ordinary email with text + HTML     -> multipart/mixed
                                       └─ multipart/alternative
                                          ├─ text/plain
                                          └─ text/html
ordinary email with attachments     -> multipart/mixed + body parts + attachments
delivery report                     -> multipart/report; report-type=<value>
meeting request/response email      -> multipart/mixed + text/calendar part
embedded message                    -> message/rfc822 part inside the parent
```

PSTD’s `pstd-eml` binary already produces deterministic plain/HTML alternative and by-value attachment paths. That is useful evidence, but parity still requires:

- raw stored header normalization before adding generated MIME headers;
- `multipart/report` with the correct report subtype and report text;
- `text/calendar` schedule parts with method and charset parameters;
- nested `message/rfc822` parts with safe header/body context;
- correct inline versus attachment disposition and Content-ID preservation;
- correct handling of messages with no plain body, only HTML, only RTF, or only an unsupported body form;
- no duplicate MIME-Version or Content-Type headers;
- stable MIME boundaries that are safe for the body and deterministic for tests.

## Charset and transfer encoding

readpst chooses a default charset from item metadata, code page, internet CPID, or `-C`. For an available UTF-8 representation it can prefer UTF-8 with `-8`; otherwise it converts UTF-8 to the selected target charset where possible. The writer uses `8bit` for suitable body text and base64 for binary attachment bytes.

PSTD should separate:

```text
source_encoding      -> how bytes were stored
decoded_text         -> a validated Unicode interpretation
output_encoding      -> what a selected adapter writes
conversion_status    -> exact, lossless, lossy, unavailable, or failed
```

The canonical archive must keep the raw body bytes and encoding metadata. An adapter may choose UTF-8, but it must not erase the source representation.

## RTF policy

readpst adds the decompressed RTF as a synthetic attachment named `rtf-body.rtf` unless `-b` is used. It can also derive HTML from Outlook’s `\fromhtml1` RTF form. PSTD should implement both explicit policies:

- preserve RTF as a body/raw artefact and optionally as a MIME attachment;
- recover HTML only when the RTF structure is validated, while retaining the original RTF;
- report decompression checksum, declared size, magic, and truncation failures;
- never classify arbitrary RTF or HTML-looking text as recovered HTML;
- allow the user to choose whether synthetic RTF output is included.

## Reports and unusual messages

Delivery-status and message-disposition reports need their own typed fields: report type, report text, report time, NDR reason/diagnostic/status codes, and supplementary information. A report should not be forced into an ordinary `multipart/mixed` message when the source class is `REPORT`.

## EML acceptance

For each supported body combination, tests must compare parsed MIME semantics rather than only strings:

- headers and address roles;
- body part media types, charset, and transfer encoding;
- body bytes after decoding;
- attachment order, names, disposition, Content-ID, and payload hashes;
- nested part ownership and boundaries;
- absence of guessed or duplicated fields.
