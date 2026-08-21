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

RP-M2-02 now publishes the header authority and charset decision before MIME
assembly. Stored Unicode and String8 header values retain raw-property evidence;
invalid or lossy values are explicitly non-authoritative. The current String8
projection records its UTF-8-lossy decoder and ISO-8859-1 fallback policy so the
later MIME adapter can implement readpst's item/code-page/`-C`/`-8` precedence
without silently changing the default output.

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

## Planned implementation — `RP-05`

### Readpst logic reviewed

`write_normal_email` chooses `multipart/report` for reports and `multipart/mixed` for ordinary messages, inserts a nested `multipart/alternative` when both text and HTML exist, emits schedule data, synthesizes decompressed RTF unless `-b`, and dispatches embedded/separate/inline attachments. `write_body_part` strips CR, chooses the selected charset, converts UTF-8 through `pst_vb_utf8to8bit` when required, detects binary/control data, and base64-encodes unsuitable text. `find_html_charset` inspects HTML metadata; `test_base64` avoids double encoding; `pst_lzfu_decompress` expands compressed RTF using its Outlook dictionary. `write_schedule_part_data` creates the calendar MIME payload. These functions operate on mutable strings and file streams, so PSTD should preserve the observable MIME semantics while using an immutable tree.

### Planned PSTD model

Add a body graph between metadata and output:

```text
BodySet {
  plain: BodyEvidence,
  html: BodyEvidence,
  rtf: BodyEvidence,
  report: BodyEvidence,
  encrypted: Vec<BodyEvidence>,
  selected: BodySelection,
}
MimeNode = Text { media_type, charset, transfer, bytes, source }
         | Multipart { subtype, params, children }
         | Attachment { attachment_key, disposition, cid }
         | Embedded { item_key }
```

Use `src/pst/messages.rs` and the current `BodyRecord`/payload maps as the evidence source. Put MIME assembly in a pure `src/output/mime.rs` (or equivalent) that accepts `BodySet`, `MessageMetadata`, and attachment/embedded keys. RTF decompression should be an isolated `src/pst/rtf.rs` adapter with declared-size/CRC/budget reporting; a future RTF-to-HTML implementation must not replace the original bytes.

### Implementation flow

1. Collect each body property independently, including raw locator bytes and the exact source tag. Classify unavailable locators instead of treating them as text.
2. Resolve charset using `RP-02` and choose a body representation through a deterministic policy (`prefer_utf8`, explicit target, or lossless raw-only).
3. Validate stored headers through `RP-04` before deciding which MIME fields are source-owned and which are generated.
4. Build the MIME tree in source/semantic order: report, ordinary mixed, alternative, schedule, embedded message, then attachments. Keep node ownership explicit.
5. Encode text using a standards-tested transfer encoder. Use `8bit` only when the target and line rules permit; otherwise use quoted-printable/base64 as declared. Never base64 an already encoded body without evidence.
6. Add RTF as a synthetic attachment when the profile requests it, with `source=rtf_compressed` and `synthetic=true`; `-b` suppresses only this projection.
7. Add encrypted text/HTML as opaque evidence with source property and payload hash. Do not call it cleartext or parse it as HTML.
8. Serialize MIME with deterministic boundaries derived from the item key and node ordinal; validate by reparsing and comparing decoded body/attachment bytes.

### Improvements over readpst

- Retain all body representations instead of mutating/deleting one while recovering another.
- Validate LZFU header sizes, magic, CRC when available, input bounds, dictionary references, and output budgets before allocating. Upstream’s `pst_lzfu_decompress` trusts `cbRawSize` and does not verify CRC.
- Use a MIME tree and ownership graph rather than interleaving `fprintf` calls, preventing duplicate MIME headers and incorrect nesting.
- Make charset and transfer decisions explicit per part, preserving raw bytes on conversion failure.
- Distinguish report text, schedule data, encrypted data, and synthetic RTF from ordinary attachments in both records and MIME.
- Correlate HTML CIDs through `RP-06` diagnostics, but never infer a match from filename coincidence.

### Issue-ready acceptance

`RP-05A` covers body evidence/selection, `RP-05B` MIME tree assembly, `RP-05C` charset/transfer encodings, `RP-05D` RTF/LZFU, `RP-05E` report/schedule parts, and `RP-05F` semantic MIME comparison. Required fixtures include plain-only, HTML-only, text+HTML, body-only RTF, generic RTF, compressed RTF, binary-looking text, malformed locators, encrypted bodies, report, schedule, and embedded MIME cases. Assertions must cover decoded bytes, media types, charsets, dispositions, CIDs, synthetic markers, boundaries, and negative statuses; update [attachments](06-attachments.md), [special items](07-embedded-and-special-email-items.md), [storage](09-storage-and-interoperability.md), and the matrix.
