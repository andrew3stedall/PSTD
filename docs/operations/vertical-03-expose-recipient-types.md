# Vertical 03: expose recipient types

## Objective

Turn the first non-internal fixed-width property observed in the public PST fixture into a semantically meaningful extraction result.

## Starting evidence

Vertical 02 changed the selected property from the table-internal `PidTagLtpRowId` to:

- property tag: `0x0c150003`;
- property identifier: `0x0c15`;
- property type: `PT_LONG`;
- raw values: `01000000`, `01000000`, `02000000`, `02000000`;
- decoded values: `1`, `1`, `2`, `2`;
- affected rows: 4.

Property identifier `0x0c15` is `PidTagRecipientType`. Its bounded values identify recipient roles: `0` originator, `1` To, `2` Cc, and `3` Bcc.

## Implementation

The Table Context property classifier now records `PidTagRecipientType` as recipient metadata rather than leaving it unknown. The fixed-width diagnostic builder uses that classification to publish:

- canonical property name `PidTagRecipientType`;
- decoded scalar values;
- semantic recipient roles;
- no semantic value for unrelated or unknown properties.

Unknown numeric recipient types remain explicit as `unknown(<value>)` rather than being guessed.

## Expected public fixture result

For the existing four-row fixture evidence, production diagnostics should now report:

- `fixed_property_name=PidTagRecipientType`;
- `fixed_semantic_values=to:to:cc:cc`.

This establishes two To-recipient rows and two Cc-recipient rows. It does not yet recover recipient names or addresses.

## Regression coverage

Tests cover:

1. authoritative classification of `0x0c15` as `PidTagRecipientType`;
2. bounded interpretation of originator, To, Cc, and Bcc values;
3. explicit handling of unknown numeric values;
4. semantic publication for validated recipient-type evidence;
5. suppression of semantic metadata for unknown properties and failed evidence.

## Extraction impact

| Measure | Before | Expected after |
|---|---:|---:|
| Messages extracted | 1 | 1 |
| Body payload records | 2 | 2 |
| Recipient rows structurally identified | 0 | 4 |
| To-recipient rows identified | 0 | 2 |
| Cc-recipient rows identified | 0 | 2 |
| Recipient names decoded | 0 | 0 |
| Recipient addresses decoded | 0 | 0 |
| Attachments extracted | 0 | 0 |
| EML files emitted | 0 | 0 |

The milestone advances recipient reconstruction without claiming that complete recipient records are readable.

## Safety boundary

- Semantic values are emitted only for validated `PidTagRecipientType` evidence.
- Raw and numeric values remain available for auditability.
- Unknown properties receive no guessed names.
- Unknown recipient-type numbers remain explicit.
- No row payload bytes are serialized.
- Message, body, and attachment extraction behavior is unchanged.

## Next vertical milestone

Use the validated recipient table rows to resolve the first recipient identity string. Prefer `PidTagDisplayName`, `PidTagEmailAddress`, or `PidTagSmtpAddress` based on descriptors actually present in the fixture. The next milestone is complete only when at least one recipient row has a readable name or address, or when fixture evidence proves that the required value is stored through an unsupported HID/HNID path.
