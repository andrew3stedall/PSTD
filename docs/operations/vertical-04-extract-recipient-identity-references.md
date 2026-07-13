# Vertical 04: extract recipient identity references

## Objective

Move from recipient-role classification to the first bounded row-level evidence needed to recover recipient names or addresses.

## Starting evidence

The public fixture contains four validated recipient rows whose `PidTagRecipientType` values are `to`, `to`, `cc`, and `cc`. Names and addresses remain unreadable because the relevant string columns are stored as four-byte HNID references rather than inline text.

## Implementation

This milestone adds bounded extraction for the authoritative recipient identity properties:

- `PidTagDisplayName` (`0x3001`);
- `PidTagEmailAddress` (`0x3003`);
- `PidTagSmtpAddress` (`0x39fe`).

Only `PT_STRING8` and `PT_UNICODE` descriptors with four-byte row storage are eligible. The extractor:

1. requires the descriptor bitmap to be set on every selected row;
2. requires the reference to remain inside the validated fixed-data region;
3. reads one `u32` HNID reference per row;
4. classifies each reference as heap ID, node ID, or null;
5. prefers display name, then SMTP address, then native email address;
6. publishes no partial result after any validation failure.

The milestone does not decode the referenced string bytes and therefore does not claim readable recipient identities yet.

## Regression coverage

Tests cover:

- mixed heap-ID and node-ID references;
- deterministic identity-property priority;
- partially absent bitmap evidence;
- rejection of non-string descriptors;
- bounded row and fixed-data validation.

## Extraction impact

| Measure | Before | After this milestone |
|---|---:|---:|
| Messages extracted | 1 | 1 |
| Recipient rows structurally identified | 4 | 4 |
| Recipient roles decoded | 4 | 4 |
| Recipient identity HNID references extractable | 0 | up to 4 per selected identity property |
| Recipient names decoded | 0 | 0 |
| Recipient addresses decoded | 0 | 0 |
| Attachments extracted | 0 | 0 |
| EML files emitted | 0 | 0 |

## Safety boundary

- No arbitrary four-byte field is interpreted as an HNID.
- Only authoritative recipient identity property identifiers and string types are accepted.
- Values absent from any selected row are not published as complete evidence.
- No referenced heap or subnode bytes are guessed or decoded in this milestone.

## Next vertical milestone

Integrate this extractor into the validated public-fixture reporting path and record the actual property, HNID values, and HNID kinds present in the four recipient rows. Then resolve the selected HNIDs through the existing heap/subnode machinery and decode the first valid `PT_UNICODE` or `PT_STRING8` value. The next milestone is complete only when at least one real recipient name or address is readable, or when the fixture proves that all selected references require a currently unsupported storage path.
