# Vertical 11: extract recipient email addresses

## Context

The public PST fixture already proved four recipient rows, their roles (`To`, `To`, `Cc`, `Cc`), and four heap-resident `PidTagDisplayName` values. The same Table Context descriptor evidence includes `PidTagEmailAddress`, while `PidTagSmtpAddress` has not been observed.

The previous recipient identity selector preferred display names. That was appropriate for proving the first readable identity, but it prevented the production pipeline from advancing to actual recipient addresses once names were already validated.

## Requirement

Prefer address-bearing recipient properties in this order:

1. `PidTagSmtpAddress` (`0x39fe`), when present on every selected row;
2. `PidTagEmailAddress` (`0x3003`), when present on every selected row;
3. `PidTagDisplayName` (`0x3001`) only as a fallback.

Retain all existing bounds, bitmap, HNID, Heap-on-Node, encoding, and fail-closed validation. Do not infer an SMTP address from a native Exchange address.

## Implementation

`extract_recipient_identity_references` now ranks SMTP address first, native email address second, and display name third. The existing end-to-end production projection therefore exercises the address property without adding another transport, diagnostic, or reporting abstraction.

Regression tests cover:

- SMTP address preferred when all three identity properties exist;
- native email address preferred when SMTP is absent;
- display name retained as the fallback;
- existing row completeness, type, size, and boundary failures.

## Public fixture acceptance evidence

GitHub Actions run `#548` completed successfully on commit `3097ec72866c9c77e3e1e137bc6513c526444b9f`.

The public fixture selected `PidTagEmailAddress` (`0x3003001f`) and resolved four heap-backed Unicode strings:

| Row | Verified role | HNID | Reference kind | Decoded address |
|---:|---|---|---|---|
| 0 | To | `0x000000e0` | `HeapId` | `to1@domain.com` |
| 1 | To | `0x00000180` | `HeapId` | `to2@domain.com` |
| 2 | Cc | `0x00000220` | `HeapId` | `cc1@domain.com` |
| 3 | Cc | `0x000002c0` | `HeapId` | `cc2@domain.com` |

The fixture reports `tc_recipient_identity_validated` with no failure reason. These values are syntactically Internet addresses and match the fixture transport headers, but they remain classified as `PidTagEmailAddress`; the parser does not relabel them as `PidTagSmtpAddress`.

## Before versus after

| Measure | Before | After |
|---|---:|---:|
| Messages discovered | 1 | 1 |
| Messages extracted | 1 | 1 |
| Body payload records | 2 | 2 |
| Recipient roles decoded | 4 | 4 |
| Recipient display names decoded | 4 | 4 |
| Readable recipient addresses | 0 | 4 |
| Complete role/name/address records emitted | 0 | 0 |
| Attachments extracted | 0 | 0 |
| EML files emitted | 0 | 0 |
| Output bytes | 35,332 | 35,371 |

The 39-byte increase reflects the shorter selected address strings replacing the prior display-name diagnostic values plus the changed property metadata. Message, body, attachment, and EML counts did not regress.

## Architectural decisions

- Reused the existing validated row selector, HNID classifier, Heap-on-Node resolver, string decoder, and production projection.
- Changed only deterministic property priority; no new transport or reporting layer was introduced.
- Required the selected property to be present on every validated row.
- Preserved `PidTagDisplayName` as a fallback so fixtures without address properties retain prior behavior.
- Kept native email and SMTP property semantics distinct.

## Remaining blockers

- Retain display names and addresses simultaneously instead of selecting only one identity property.
- Assemble role, display name, and address by the same validated row index.
- Publish structured recipients in message output and eventual EML headers.
- Normalize HTML and RTF bodies.
- Extract attachment tables, payloads, and embedded messages.
- Emit complete EML files and validate against broader fixtures.

## Following vertical milestone

Resolve display name and email address in parallel and emit four structured recipient records while preserving row alignment:

```text
To: Recipient 1 <to1@domain.com>
To: Recipient 2 <to2@domain.com>
Cc: Recipient 3 <cc1@domain.com>
Cc: Recipient 4 <cc2@domain.com>
```

The next milestone must modify production message output, not add another standalone selector, wrapper, formatter, or diagnostic type.
