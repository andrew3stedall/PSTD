# Vertical 20: Select one message recipient projection

## Objective

Establish the final fail-closed attribution boundary between a message-level Table Context probe and structured recipient output.

## Context

Vertical 19 retained each typed `TcCompleteRecipientProjectionReport` on its resolved heap diagnostic. The message extraction loop can now receive the attributed probe, but emitting recipients immediately still requires a rule for zero, one, or multiple validated recipient-table candidates.

Combining multiple tables or choosing the first table would be unsafe. The parser must require exactly one validated, non-empty complete-recipient projection for a message.

## Implementation

Added `tc_message_recipient_selection` with `select_message_recipient_projection`.

The selector:

- examines only typed `complete_recipients` evidence on the message-attributed probe;
- accepts reports whose complete records are validated and non-empty;
- returns the complete record report only when exactly one candidate exists;
- reports unavailable evidence when no validated candidate exists;
- reports ambiguity and exposes no records when multiple validated candidates exist;
- never parses diagnostic status text;
- never combines recipient rows from multiple Table Context heaps.

## Regression coverage

Tests verify:

- one validated report is selected even when failed evidence is also present;
- zero validated reports remain unavailable;
- multiple validated reports fail closed without exposing partial records.

## Extraction impact

This milestone does not yet change public-fixture output. It closes the exactly-one-table attribution requirement needed before appending recipient records to `MetadataExtractionOutput.recipients`.

Confirmed fixture baseline remains:

- one extracted message;
- four recipient roles;
- four display names;
- four native email-address values;
- four complete recipient records in typed diagnostic evidence;
- zero structured recipient output records.

## Next milestone

At the message extraction call site:

1. retain the `TcSubnodeProbeReport` returned by `record_subnode_payload_probe`;
2. call `select_message_recipient_projection`;
3. convert the selected complete record report through `message_recipients_from_complete_records` using the matching message key;
4. append the four converted records to `MetadataExtractionOutput.recipients`;
5. preserve fail-closed output for unavailable, ambiguous, or invalid evidence.

Acceptance must require four structured recipient records in one public-fixture run with stable message, body, attachment, and byte-count metrics.
