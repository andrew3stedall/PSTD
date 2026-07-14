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
- removes non-RFC control bytes from header values after rejecting CR/LF injection;
- emits one file per message that satisfies the complete boundary.

HTML/RTF alternatives, attachments, embedded messages, and inferred address semantics are excluded.

## Public fixture result

One EML was emitted for `msg_ad9f58792ae34dfc` with these exact readable headers:

```text
From: Sender Name <from@domain.com>
To: Recipient 1 <to1@domain.com>, Recipient 2 <to2@domain.com>
Cc: Recipient 3 <cc1@domain.com>, Recipient 4 <cc2@domain.com>
Subject: New message created by Aspose.Email for Java(Aspose.Email Evaluation)
MIME-Version: 1.0
Content-Type: text/plain; charset=utf-8
Content-Transfer-Encoding: 8bit
```

The UTF-8 plain-text body is 214 bytes before CRLF normalization and begins:

```text
This is an evaluation copy of Aspose.Email for Java
```

The final EML is 574 bytes and contains no bare LF line endings or non-whitespace C0 controls in its headers.

## Before versus after

| Measure | Before | After |
|---|---:|---:|
| Messages extracted | 1 | 1 |
| Body payload records | 2 | 2 |
| Structured recipient records | 4 | 4 |
| Attachments extracted | 0 | 0 |
| EML files emitted | 0 | 1 |
| Structured extraction output bytes | 40,722 | 40,722 |
| EML bytes | 0 | 574 |
| Combined observable output bytes | 40,722 | 41,296 |

## Safety boundary

The serializer does not reinterpret PST bytes. It assembles only typed extraction results from the existing parser. A message is skipped unless all required components are present and valid.

## Remaining limitations

- the EML command is a dedicated assembly binary rather than part of the main archive writer;
- no Date header is emitted until timestamp formatting and source preference are validated;
- HTML/RTF alternatives are not selected;
- attachments and embedded messages remain unavailable;
- non-ASCII header values are not yet RFC 2047 encoded.
