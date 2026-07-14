# Vertical 23: emit the first readable EML

## Objective

Produce one readable `.eml` file for the public PST fixture by assembling fields that are already validated and attached to the extracted message.

## Scope

This slice adds a bounded `pstd-eml` binary that:

- runs the existing metadata extraction path;
- requires a validated subject and sender address;
- requires at least one structured To or Cc recipient;
- selects the existing non-empty UTF-8 plain-text body;
- renders deterministic CRLF headers and body separation;
- rejects header injection, missing recipients, missing body data, and unknown body encoding rather than guessing;
- emits one file per message that satisfies the complete boundary.

HTML/RTF alternatives, attachments, embedded messages, and inferred address semantics are excluded.

## Public fixture acceptance

The fixture run must emit exactly one EML containing:

```text
To: Recipient 1 <to1@domain.com>, Recipient 2 <to2@domain.com>
Cc: Recipient 3 <cc1@domain.com>, Recipient 4 <cc2@domain.com>
```

It must also contain a non-empty From header, Subject header, UTF-8 plain-text MIME declaration, non-empty body, and no bare LF line endings.

## Before

- messages extracted: 1
- body payload records: 2
- structured recipient records: 4
- attachments extracted: 0
- EML files emitted: 0
- output bytes: 40,722

## After

Exact subject, sender, body evidence, EML byte count, and aggregate before-versus-after output bytes are recorded from the exact-head Actions artifact before merge.

## Safety boundary

The serializer does not reinterpret PST bytes. It assembles only typed extraction results from the existing parser. A message is skipped unless all required components are present and valid.
