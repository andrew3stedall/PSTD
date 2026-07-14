# Vertical 25: expose a readable RTF body

## Objective

Decode the real `PidTagRtfCompressed` payload already extracted from the public PST and expose it as a valid standalone RTF document.

## Scope

This slice operates directly on existing `BodyPayload` bytes whose body type is `rtf`. It validates the 16-byte compressed-RTF header, compressed-size boundary, CRC-32, compression magic, declared uncompressed size, and final RTF signature.

Supported containers:

- `LZFu`: dictionary-based compressed RTF;
- `MELA`: uncompressed RTF wrapped in the same validated header.

Malformed, truncated, unknown-magic, CRC-invalid, size-mismatched, or non-RTF output is rejected without producing a partial file.

## Observable acceptance

The public fixture must emit exactly one `.rtf` file associated with the extracted message. The decoded document must begin with `{\\rtf` and contain the already known fixture body text:

```text
This is an evaluation copy of Aspose.Email for Java
```

The exact decoded byte count is recorded from the workflow artifact after CI succeeds.

## Before versus after

| Measure | Before | Expected after |
|---|---:|---:|
| Messages extracted | 1 | 1 |
| Body payload records | 2 | 2 |
| Structured recipient records | 4 | 4 |
| Attachments extracted | 0 | 0 |
| EML files emitted | 1 | 1 |
| Readable standalone RTF files | 0 | 1 |

## Next decision

After the fixture RTF is decoded and measured, compare its readable content with the plain-text body. If it adds useful fidelity, integrate it into the existing EML as a standards-compliant MIME alternative. If it is semantically duplicate, move to the smallest body or attachment slice that exposes new content rather than adding another RTF wrapper.
