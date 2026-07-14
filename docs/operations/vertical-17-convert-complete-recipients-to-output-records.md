# Vertical 17: Convert complete recipients to output records

_Last reviewed: 14 July 2026._

## Context

Vertical 16 proved that one production public-fixture run can project four complete recipient rows containing role, display name, address, and authoritative address kind. The production reporting path currently serializes that evidence into a diagnostic status fragment, while the extraction output already defines `RecipientRecord` as the structured message-recipient contract.

Parsing the diagnostic string back into output records would couple extraction correctness to formatting and could silently mis-handle delimiters. This milestone therefore establishes a typed conversion boundary from validated `TcCompleteRecipientRecordReport` evidence into `RecipientRecord` values.

## Requirement revision

The previous roadmap wording suggested attaching recipients directly to `MetadataExtractionOutput` in one step. Repository inspection showed that the typed complete-recipient report is not yet retained at the message extraction call site; only its formatted status is retained. Direct attachment without first defining a typed conversion would either duplicate mapping logic inside the large metadata loop or parse the diagnostic string.

This vertical slice is intentionally limited to the conversion required by the next production integration. It does not add another parser, projection, or diagnostic representation.

## Implementation

Added `tc_message_recipients` with `message_recipients_from_complete_records`.

The conversion:

- accepts only `tc_recipient_records_validated` evidence;
- requires a non-empty, contiguous row sequence starting at zero;
- preserves To, Cc, Bcc, and originator role semantics;
- preserves display name and raw address values;
- sets `smtp_address` only when the authoritative address kind is `smtp_address`;
- does not infer SMTP merely because a native address contains `@`;
- creates deterministic recipient keys through the existing output ID helper;
- fails closed with no partial `RecipientRecord` values for invalid roles, row ordering, empty fields, or unknown address kinds.

## Public-fixture-shaped result

The validated fixture evidence converts to four output records:

| Ordinal | Type | Display name | Raw address | SMTP address |
|---:|---|---|---|---|
| 0 | to | Recipient 1 | `to1@domain.com` | none |
| 1 | to | Recipient 2 | `to2@domain.com` | none |
| 2 | cc | Recipient 3 | `cc1@domain.com` | none |
| 3 | cc | Recipient 4 | `cc2@domain.com` | none |

The SMTP field remains empty because the fixture property is `PidTagEmailAddress`, classified as `native_email_address`. This avoids promoting a format-based guess into semantic output.

## Regression coverage

Tests cover:

- conversion of the exact four-row fixture shape;
- deterministic role, ordinal, raw-address, and key mapping;
- authoritative SMTP handling;
- failure for non-contiguous rows;
- suppression of partial output when source evidence is failed or unavailable.

## Remaining production boundary

The next milestone must retain the typed complete-recipient report on the message-attributed Table Context probe and call this conversion from `extract_metadata` for the matching message key. It must then append all four records to `MetadataExtractionOutput.recipients` in the same run.

Acceptance for the following run:

- four structured recipient JSONL records are emitted for the public message;
- all records carry the correct message key, role, name, address, address kind mapping, and ordinal;
- incomplete Table Context evidence emits no partial recipients;
- existing diagnostic, message, body, attachment, and archive output remains stable;
- full CI and public-fixture evidence pass on the exact head.
