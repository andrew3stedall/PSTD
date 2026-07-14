# Vertical 21: Build message recipient output

## Context

Vertical 20 added the exactly-one-recipient-table selection rule. The repository already had a typed conversion from validated complete recipient records into the output `RecipientRecord` contract, but the two operations were still separate. Calling them independently at the extraction site would make it easier to accidentally bypass the ambiguity rule or expose partial conversion results.

## Revised requirement

The next safe step is to provide one fail-closed operation that consumes a message-attributed Table Context probe and returns structured recipient output only when:

1. exactly one validated, non-empty complete recipient projection is attributed to the message; and
2. every selected row converts successfully into the existing output contract.

This milestone deliberately does not parse diagnostic strings, merge multiple tables, infer SMTP from address shape, or emit partial recipients.

## Implementation

`tc_message_recipient_output::build_message_recipient_output` now composes the existing validated boundaries:

- `select_message_recipient_projection` enforces exactly one candidate;
- `message_recipients_from_complete_records` validates row order, roles, display names, addresses, and authoritative address kind;
- unavailable, ambiguous, or invalid evidence returns an empty recipient list and a bounded failure reason.

The report retains candidate count so the extraction call site can distinguish no table from ambiguous attribution without inspecting presentation text.

## Regression evidence

Focused tests verify:

- one selected native-address record becomes a structured `RecipientRecord` with the correct message key;
- native `PidTagEmailAddress` evidence remains raw and is not promoted to SMTP;
- ambiguous selection exposes no partial recipients.

## Extraction impact

No new public-fixture field is claimed in this vertical. The confirmed baseline remains:

- one extracted message;
- four recipient roles;
- four display names;
- four native email-address values;
- four complete typed recipient records available on the attributed probe;
- zero structured recipient output records until the extraction loop appends this report.

## Following run

The next vertical should modify `extract_metadata` to retain the report returned by `record_subnode_payload_probe`, call `build_message_recipient_output` with the matching message key, and append recipients only when the report status is `tc_message_recipient_output_attached`.

Acceptance must require four recipient records in the public fixture output with existing message, body, attachment, and byte-count metrics unchanged. Unavailable or ambiguous evidence must remain empty and should be surfaced as message-level status evidence rather than guessed.
