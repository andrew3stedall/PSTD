# Vertical 25: expose a readable RTF body

## Objective

Decode the real `PidTagRtfCompressed` payload already extracted from the public PST and expose it as a valid standalone RTF document.

## Scope

This slice operates directly on existing `BodyPayload` bytes whose body type is `rtf`. It validates the 16-byte compressed-RTF header, compressed-size boundary, container magic, declared framing size, and final RTF signature.

Supported containers:

- `LZFu`: dictionary-based compressed RTF with validated CRC-32;
- `MELA`: uncompressed RTF with a zero CRC field and matching framed-size fields.

Malformed, truncated, unknown-magic, invalid-CRC, size-mismatched, or non-RTF output is rejected without producing a partial file.

## Public fixture evidence

The public fixture contains one 336-byte `PidTagRtfCompressed` value:

- compressed/framed size: 332 bytes;
- raw/framed size: 332 bytes;
- magic: `MELA` (`0x414c454d`);
- CRC field: `0x00000000`;
- decoded standalone RTF: 320 bytes;
- message key: `msg_ad9f58792ae34dfc`.

The decoded document begins with `{\\rtf1`, identifies itself as HTML-derived RTF with `\\fromhtml1`, and exposes rich-text content not present in the existing plain-text body:

```text
This line is in bold.
This line is in blue color
```

## Before versus after

| Measure | Before | After |
|---|---:|---:|
| Messages extracted | 1 | 1 |
| Body payload records | 2 | 2 |
| Structured recipient records | 4 | 4 |
| Attachments extracted | 0 | 0 |
| EML files emitted | 1 | 1 |
| Readable standalone RTF files | 0 | 1 |
| Readable standalone RTF bytes | 0 | 320 |
| EML bytes | 613 | 613 |

## Next vertical milestone

Integrate the validated 320-byte rich-text representation into the existing EML as a standards-compliant alternative while retaining the current plain-text body. The next slice must produce an observable multipart EML and must not add another RTF wrapper or diagnostic-only layer.
