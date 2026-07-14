# Vertical 22: emit structured message recipients

## Objective

Attach the already validated complete recipient projection directly to `MetadataExtractionOutput.recipients` during the real message extraction pass. This is the first recipient milestone that changes the structured output contract rather than adding another parser or reporting boundary.

## Scope

- Retain the typed `TcSubnodeProbeReport` returned by the existing message-attributed probe.
- Invoke `build_message_recipient_output` once for the message and the already decoded subnode payloads.
- Append recipients only when the output status is `tc_message_recipient_output_attached`.
- Apply the same fail-closed rule to messages with and without attachment properties.
- Do not parse diagnostic strings, guess SMTP semantics, or decode the recipient table a second time.

## Public fixture acceptance

The public PST run must emit four structured recipient JSONL records for one message:

```text
To: Recipient 1 <to1@domain.com>
To: Recipient 2 <to2@domain.com>
Cc: Recipient 3 <cc1@domain.com>
Cc: Recipient 4 <cc2@domain.com>
```

`PidTagEmailAddress` values remain native addresses. The `smtp_address` field must remain empty unless `PidTagSmtpAddress` is the validated source property.

## Before baseline

| Measure | Before |
|---|---:|
| Messages extracted | 1 |
| Body payload records | 2 |
| Structured recipient records | 0 |
| Attachments extracted | 0 |
| EML files emitted | 0 |
| Output bytes | 39,622 |

## After evidence

To be replaced with exact GitHub Actions public-fixture artifact values before merge.

## Failure behaviour

Zero, ambiguous, invalid, or incomplete recipient projections append no records. No partial recipient list is emitted.

## Next vertical milestone

Once the four structured records are fixture-visible, use those records to emit real `To` and `Cc` headers in the first EML output rather than adding another recipient transport or formatting layer.
