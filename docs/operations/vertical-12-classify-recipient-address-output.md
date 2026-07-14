# Vertical 12: classify recipient address output

## Context

CI run 549 for PR 427 completed successfully and the public PST fixture selected `PidTagEmailAddress` (`0x3003001f`). It decoded four heap-resident values:

- `to1@domain.com`
- `to2@domain.com`
- `cc1@domain.com`
- `cc2@domain.com`

The existing diagnostic exposed those strings through the generic `recipient_values` field, but did not state whether the selected property represented an SMTP address, a native Exchange address, or a display name.

## Requirement revision

The previous proposal was to retain names and addresses concurrently and assemble complete recipient records. That remains the correct larger objective, but it is too broad for one safe change because the current projection selects exactly one identity property.

This milestone makes the newly extracted address values semantically explicit without mislabelling `PidTagEmailAddress` as SMTP. It adds no parser transport or storage abstraction.

## Change

`TcRecipientIdentityDiagnostic::status_fragment` now publishes `recipient_value_kind`:

- `smtp_address` for `PidTagSmtpAddress`
- `native_email_address` for `PidTagEmailAddress`
- `display_name` for `PidTagDisplayName`
- `unknown` for any other named property
- `none` when no validated property is available

The public fixture is expected to publish:

```text
recipient_property_name=PidTagEmailAddress,
recipient_value_kind=native_email_address,
recipient_values=to1@domain.com:to2@domain.com:cc1@domain.com:cc2@domain.com
```

## Safety

- no address transformation is performed;
- native email addresses are not relabelled as SMTP addresses;
- unavailable and failed projections publish `recipient_value_kind=none`;
- row and heap payload bytes remain excluded from status output;
- existing fail-closed extraction behavior is unchanged.

## Extraction progress

The public fixture now has four validated address-bearing strings and four independently validated recipient roles. They are not yet assembled into production recipient records, and display names are not retained concurrently with addresses.

## Following milestone

The next vertical milestone should resolve both `PidTagDisplayName` and the preferred address property from the same validated rows, then assemble row-aligned records containing:

```text
role + display_name + address + address_kind
```

Completion requires the four fixture rows to be published as complete recipient records without relying on positional data from separate runs.
