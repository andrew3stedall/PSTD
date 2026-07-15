# Vertical 26: emit multipart plain-text and RTF EML

## Objective

Emit one readable `multipart/alternative` EML for the public PST fixture using the already validated plain-text body and decoded RTF body. This milestone changes message output directly; it does not add another parser abstraction or diagnostic layer.

## Extraction boundary

The EML command groups existing `BodyPayload` records by message, requires one non-empty UTF-8 plain-text body and one validated RTF body, and assembles them as deterministic MIME alternatives. The RTF path reuses the validated `MELA` and `LZFu` rules established by Vertical 25. It does not re-traverse or reinterpret PST structures.

Emission fails closed when required sender, subject, recipient, plain-text, or RTF evidence is absent; when RTF validation fails; when either body is not UTF-8; or when body content collides with the deterministic MIME boundary.

## Observable fixture result

The public fixture emits exactly one 1,175-byte EML with:

- root content type `multipart/alternative`;
- first part `text/plain; charset=utf-8`, 232 decoded bytes, containing `This is an evaluation copy of Aspose.Email for Java`;
- second part `text/rtf; charset=utf-8`, 320 decoded bytes, beginning with `{\\rtf`;
- rich content `This line is in bold.` and `This line is in blue color`;
- sender `Sender Name <from@domain.com>`;
- To recipients `Recipient 1 <to1@domain.com>, Recipient 2 <to2@domain.com>`;
- Cc recipients `Recipient 3 <cc1@domain.com>, Recipient 4 <cc2@domain.com>`;
- subject `New message created by Aspose.Email for Java(Aspose.Email Evaluation)`;
- Date `Wed, 19 Aug 2015 11:07:26 +0000`;
- deterministic CRLF line endings and a closed deterministic boundary.

Exact-head GitHub Actions results on `df13917c4ff76f8863b7f2cb18e0db0fad174aa3` were CI #614, Readable EML fixture #24, and Readable RTF fixture #13; all passed.

## Before versus after

| Measure | Before | After |
|---|---:|---:|
| Messages extracted | 1 | 1 |
| Body payload records | 2 | 2 |
| Structured recipients | 4 | 4 |
| Attachments extracted | 0 | 0 |
| EML files | 1 | 1 |
| EML body representations | 1 | 2 |
| Plain-text MIME parts | 1 | 1 |
| RTF MIME parts | 0 | 1 |
| Standalone readable RTF files | 1 | 1 |
| Structured extraction bytes | 40,722 | 40,722 |
| EML bytes | 613 | 1,175 |
| Standalone RTF bytes | 320 | 320 |
| Combined observable bytes | 41,655 | 42,217 |

The 562-byte EML increase is the MIME framing and inclusion of the validated rich-text representation. Message, body-record, recipient, attachment, and standalone RTF counts did not regress.

## Next decision

The next vertical milestone should inspect attachment-table evidence and recover one real attachment field or payload. Further MIME wrappers, diagnostics, or body-reporting layers are not justified.