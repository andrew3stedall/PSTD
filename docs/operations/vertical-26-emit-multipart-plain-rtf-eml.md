# Vertical 26: emit multipart plain-text and RTF EML

## Objective

Emit one readable `multipart/alternative` EML for the public PST fixture using the already validated plain-text body and decoded RTF body. This milestone changes message output directly; it does not add another parser abstraction or diagnostic layer.

## Extraction boundary

The EML command groups existing `BodyPayload` records by message, requires one non-empty UTF-8 plain-text body and one validated RTF body, and assembles them as deterministic MIME alternatives. The RTF path reuses the validated `MELA` and `LZFu` rules established by Vertical 25. It does not re-traverse or reinterpret PST structures.

Emission fails closed when required sender, subject, recipient, plain-text, or RTF evidence is absent; when RTF validation fails; when either body is not UTF-8; or when body content collides with the deterministic MIME boundary.

## Observable fixture acceptance

The public fixture must emit exactly one EML with:

- root content type `multipart/alternative`;
- first part `text/plain; charset=utf-8` containing `This is an evaluation copy of Aspose.Email for Java`;
- second part `text/rtf; charset=utf-8` beginning with `{\\rtf`;
- rich content `This line is in bold.` and `This line is in blue color`;
- the existing sender, To, Cc, Subject, Date, and Message-ID fields;
- deterministic CRLF line endings and a closed deterministic boundary.

## Before versus after target

| Measure | Before | After acceptance |
|---|---:|---:|
| Messages extracted | 1 | 1 |
| Body payload records | 2 | 2 |
| Structured recipients | 4 | 4 |
| Attachments extracted | 0 | 0 |
| EML files | 1 | 1 |
| EML body representations | plain text only | plain text and RTF |
| Standalone readable RTF files | 1 | 1 |

Exact multipart EML bytes are recorded from the workflow artifact after the first green fixture run.

## Next decision

After this EML is proven against the fixture, the next vertical milestone should inspect attachment-table evidence and recover one real attachment field or payload. Further MIME wrappers, diagnostics, or body-reporting layers are not justified.
