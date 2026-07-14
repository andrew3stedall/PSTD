# Vertical 22: emit structured message recipients

## Objective

Attach the already validated complete recipient projection directly to `MetadataExtractionOutput.recipients` during the real message extraction pass. This is the first recipient milestone that changes structured message output rather than adding another parser or reporting boundary.

## Scope

- Retain the typed `TcSubnodeProbeReport` returned by the existing message-attributed probe.
- Invoke `build_message_recipient_output` once for the message and the already decoded subnode payloads.
- Append recipients only when the output status is `tc_message_recipient_output_attached`.
- Apply the same fail-closed rule to messages with and without attachment properties.
- Do not parse diagnostic strings, guess SMTP semantics, or decode the recipient table a second time.
- Assert the exact structured records in the public-PST CI fixture and retain `recipients.jsonl` in the evidence artifact.

## Public fixture result

GitHub Actions run 587 emitted four structured recipient records for message `msg_ad9f58792ae34dfc`:

```text
To: Recipient 1 <to1@domain.com>
To: Recipient 2 <to2@domain.com>
Cc: Recipient 3 <cc1@domain.com>
Cc: Recipient 4 <cc2@domain.com>
```

The exact structured values are:

| Ordinal | Role | Display name | Raw address | Address type | SMTP address |
|---:|---|---|---|---|---|
| 0 | `to` | `Recipient 1` | `to1@domain.com` | `native_email_address` | null |
| 1 | `to` | `Recipient 2` | `to2@domain.com` | `native_email_address` | null |
| 2 | `cc` | `Recipient 3` | `cc1@domain.com` | `native_email_address` | null |
| 3 | `cc` | `Recipient 4` | `cc2@domain.com` | `native_email_address` | null |

`PidTagEmailAddress` remains a native address. The implementation does not infer SMTP semantics from the value format.

## Before versus after

| Measure | Before | After |
|---|---:|---:|
| Messages discovered | 1 | 1 |
| Messages extracted | 1 | 1 |
| Body payload records | 2 | 2 |
| Structured recipient records | 0 | 4 |
| Attachments extracted | 0 | 0 |
| EML files emitted | 0 | 0 |
| Output bytes | 39,622 | 40,722 |

The 1,100-byte increase is exactly the four-line `recipients.jsonl` payload retained in the CI artifact. No body, attachment, or EML count changed.

## Regression evidence

CI run 587 passed Rust build and tests, Clippy, rustfmt, Python checks, Docker build, CLI checks, the public-PST extraction, and an exact assertion over all four recipient records.

## Failure behaviour

Zero, ambiguous, invalid, or incomplete recipient projections append no records. No partial recipient list is emitted.

## Next vertical milestone

Generate the first EML file from the already readable message metadata, structured recipients, and plain-text body. The next change must produce an actual `.eml` artifact rather than another recipient transport or formatting layer.
