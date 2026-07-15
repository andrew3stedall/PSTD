# Vertical 28: emit plain-text and HTML EML alternatives

## Objective

Replace the public fixture EML's raw `text/rtf` alternative with the already validated HTML recovered from the fixture's `\fromhtml1` RTF body, while retaining the existing plain-text representation and validated message headers.

This milestone changes the observable email output directly. It does not add a parser transport, diagnostic wrapper, or second interpretation of the PST property bytes.

## Before

- messages extracted: 1
- body payload records: 2 (`text`, `rtf`)
- structured recipients: 4
- attachments: 0
- EML files: 1
- EML root type: `multipart/alternative`
- EML alternatives: `text/plain`, `text/rtf`
- EML bytes: 1,175
- standalone RTF files: 1
- standalone RTF bytes: 320
- standalone HTML files: 1
- standalone HTML bytes: 95
- combined EML, RTF, and HTML bytes: 1,590

## Implementation boundary

The existing `pstd-eml` command now:

- reuses the validated MELA/LZFu RTF decoder;
- requires a readable `\fromhtml1` RTF document;
- recovers HTML markup carried by `\htmltag` destinations;
- retains visible body text;
- skips RTF metadata, picture, object, font, colour, and stylesheet destinations;
- emits ordered `text/plain` then `text/html` MIME alternatives;
- rejects malformed groups, unsafe header values, leaked RTF controls, missing HTML, missing plain text, and MIME-boundary collisions;
- emits deterministic CRLF output.

The public-fixture workflow parses the generated message with Python's standard email parser and requires exactly the two expected MIME parts. It also asserts the deterministic output size.

## Public fixture result

The fixture emitted exactly one HTML-backed EML for message `msg_ad9f58792ae34dfc`.

```text
multipart/alternative
├── text/plain; charset=utf-8
└── text/html; charset=utf-8
```

Exact output evidence:

- EML bytes: 956
- decoded plain-text MIME bytes: 232
- decoded HTML MIME bytes after CRLF normalization: 100
- sender: `Sender Name <from@domain.com>`
- To: `Recipient 1 <to1@domain.com>, Recipient 2 <to2@domain.com>`
- Cc: `Recipient 3 <cc1@domain.com>, Recipient 4 <cc2@domain.com>`
- Date: `Wed, 19 Aug 2015 11:07:26 +0000`

Recovered HTML content includes:

```html
<b>This line is in bold.
</b> 
<br/> 
<br/>
<font color=blue>This line is in blue color
</font>
```

The HTML part contains no raw `\rtf` or `\htmltag` controls and the EML contains no `text/rtf` MIME part.

## After

- messages extracted: 1
- body payload records: 2 (`text`, `rtf`)
- structured recipients: 4
- attachments: 0
- EML files: 1
- EML root type: `multipart/alternative`
- EML alternatives: `text/plain`, `text/html`
- EML bytes: 956
- standalone RTF files: 1
- standalone RTF bytes: 320
- standalone HTML files: 1
- standalone HTML bytes: 95
- combined EML, RTF, and HTML bytes: 1,371

The EML decreased by 219 bytes because the 320-byte RTF alternative was replaced by the smaller recovered HTML representation and corresponding MIME metadata. No readable content, message, recipient, body, or attachment count regressed.

## Verification

- Rust tests: passed.
- Clippy with warnings denied: passed.
- rustfmt: passed.
- readable RTF and HTML fixture: passed.
- readable EML fixture: passed and asserted `html_backed_eml_bytes=956`.
- generated artifact parsed as ordered `text/plain` and `text/html` alternatives.

## Next vertical milestone

The current approved public fixture contains zero attachment candidates. Do not add attachment-only infrastructure against it. The next extraction milestone should use an approved PST fixture containing at least one real attachment or broaden observable validation across additional real messages and PST layouts.
