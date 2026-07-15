# Vertical 27: recover readable HTML from validated RTF

## Objective

Recover an observable HTML representation from the public fixture's already validated 320-byte `\fromhtml1` RTF body. This milestone does not reinterpret PST bytes or add another parser transport layer; it converts the validated rich-body artifact into a useful body representation.

## Before

- messages extracted: 1
- body payload records: 2 (`text`, `rtf`)
- structured recipients: 4
- attachments: 0
- EML files: 1
- EML bytes: 1,175
- standalone RTF files: 1
- standalone RTF bytes: 320
- standalone HTML files: 0
- standalone HTML bytes: 0

## Implementation boundary

`scripts/rtf_fromhtml_to_html.py` accepts only a readable RTF document that begins with `{\rtf` and declares `\fromhtml1`. It:

- recovers markup carried by `\htmltag` destinations;
- retains visible body text;
- skips RTF metadata destinations such as font, colour, stylesheet, object, and picture tables;
- handles escaped braces, backslashes, tabs, line breaks, and hex bytes;
- rejects unbalanced groups, invalid escapes, non-`fromhtml1` input, markup-free output, and leaked RTF control data.

The public-fixture workflow first invokes the existing validated `pstd-rtf` decoder, then runs HTML recovery against that exact output. This reuses the validated compressed-RTF component rather than duplicating PST or LZFu/MELA decoding.

## Public fixture acceptance

The fixture must emit exactly one RTF file and one HTML file for message `msg_ad9f58792ae34dfc`. The HTML must contain:

```html
<b>This line is in bold.</b>
<font color=blue>This line is in blue color</font>
```

It must contain no raw `\rtf` or `\htmltag` controls. Exact HTML bytes are recorded by the workflow artifact and must be copied into this evidence document before merge.

## After

Pending exact-head fixture execution. The branch must remain unmerged until the recovered HTML values, byte count, tests, and all GitHub Actions checks are green.

## Next vertical milestone

After this milestone, integrate the validated HTML representation into the EML as the preferred rich alternative while retaining plain text. Attachment work remains blocked on the current public fixture because it contains zero attachment candidates.
