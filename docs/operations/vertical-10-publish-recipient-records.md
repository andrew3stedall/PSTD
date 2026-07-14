# Vertical 10: publish recipient records

## Starting evidence

PR 425 merged the fail-closed assembly of independently validated recipient-role and display-name sequences.

The public fixture evidence available from the preceding production runs is:

- recipient roles: `to, to, cc, cc`
- recipient display names: `Recipient 1, Recipient 2, Recipient 3, Recipient 4`
- structured records: four row-aligned role/name pairs
- recipient addresses: not yet decoded

## Milestone

Add a stable, bounded publication format for assembled recipient records.

Validated evidence is represented as:

```text
recipient_records_status=tc_recipient_records_validated;
recipient_records=0:to:Recipient 1|1:to:Recipient 2|2:cc:Recipient 3|3:cc:Recipient 4;
recipient_records_failure=none
```

The formatter:

- preserves row order;
- publishes no raw row or heap payload bytes;
- sanitizes delimiters in roles, identities, and failure reasons;
- emits `recipient_records=none` after unavailable or failed assembly;
- preserves fail-closed row-count mismatch behavior.

## Extraction impact

This makes the four validated role/name recipient records safe to expose through reporting surfaces. It does not yet add email addresses or attach recipients to emitted EML messages.

Current public-fixture baseline remains:

- folders discovered: 11
- messages extracted: 1
- body payload records: 2
- recipient rows: 4
- recipient role/name records: 4
- recipient addresses: 0
- attachments extracted: 0
- bytes written: 35,332

## Evidence required from CI

The pull request must pass formatting, compilation, unit tests, clippy, wrapper tests, Docker build, and the public-PST fixture before merge.

## Next vertical milestone

Connect the bounded recipient-record fragment to the existing production `TcHeapDiagnostic` progress output, then extract `PidTagEmailAddress` (`0x3003`) from the same four rows. The next extraction milestone is complete only when at least one real address is associated with its validated role/name record, or exact fixture evidence proves a different storage path is required.
