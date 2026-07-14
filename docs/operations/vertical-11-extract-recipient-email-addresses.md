# Vertical 11: extract recipient email addresses

## Context

The public PST fixture already proves four recipient rows, their roles (`To`, `To`, `Cc`, `Cc`), and four heap-resident `PidTagDisplayName` values. The same Table Context descriptor evidence also includes `PidTagEmailAddress`, while `PidTagSmtpAddress` has not been observed.

The previous recipient identity selector preferred display names. That was appropriate for proving the first readable identity, but it prevented the production pipeline from advancing to actual recipient addresses once names were already validated.

## Revised requirement

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

## Acceptance evidence required from CI

The public-PST fixture must report one of the following on the exact PR head:

- `PidTagEmailAddress` with four decoded row values;
- `PidTagSmtpAddress` with four decoded row values; or
- a bounded failure proving why the observed address property cannot be decoded.

The milestone must not claim four usable Internet addresses merely because four strings decode. Native Exchange legacy distinguished names remain native addresses unless an SMTP property or authoritative conversion evidence is available.

## Baseline before this change

- messages extracted: 1;
- recipient rows: 4;
- recipient roles: two To, two Cc;
- recipient display names: four;
- recipient addresses: zero confirmed;
- attachments: zero;
- EML output: zero.

## Following vertical milestone

If four address strings validate, retain display name and address simultaneously and assemble role/name/address recipient records for message output. If the values are native Exchange addresses, the next milestone should search the same rows for `PidTagSmtpAddress` or a validated address-book mapping rather than relabelling native values as SMTP.
