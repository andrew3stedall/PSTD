# Vertical 24: emit a validated Date header

## Objective

Add one authoritative, observable timestamp to the readable public-fixture EML without guessing from absent structured timestamp fields.

## Scope

The fixture message has no populated `sent_at`, `received_at`, `created_at`, or `modified_at` value. Its validated transport-header block contains exactly one Date field:

```text
Date: 19 Aug 2015 11:07:26 +0000
```

This slice:

- selects Date only from the already extracted transport-header block;
- requires exactly one unfolded Date field;
- parses it with RFC 2822 semantics;
- emits a canonical RFC-style EML header;
- omits Date when the source is absent, duplicated, malformed, or contains header injection;
- preserves all existing sender, recipient, subject, body, and EML behaviour.

It does not infer a delivery or submission timestamp, and it does not relabel the transport Date as a MAPI property.

## Expected public fixture result

```text
Date: Wed, 19 Aug 2015 11:07:26 +0000
```

The exact EML byte count and final workflow result will be recorded from the pull-request artifact before merge.

## Before versus after

| Measure | Before | Expected after |
|---|---:|---:|
| Messages extracted | 1 | 1 |
| Body payload records | 2 | 2 |
| Structured recipient records | 4 | 4 |
| Attachments extracted | 0 | 0 |
| EML files emitted | 1 | 1 |
| EML Date headers | 0 | 1 |
| Structured extraction bytes | 40,722 | 40,722 |
| EML bytes | 574 | pending fixture run |

## Safety boundary

A malformed or ambiguous transport Date does not prevent an otherwise readable EML from being emitted; it only suppresses the optional Date header. This avoids inventing time semantics while preserving the existing complete message output.

## Next decision

After the validated Date is emitted, the next vertical milestone should be chosen from fixture evidence. The likely priority is HTML/RTF body handling or broader fixture validation; attachment work should not be selected solely because it appears later in a predetermined sequence.
