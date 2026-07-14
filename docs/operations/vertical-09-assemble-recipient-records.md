# Vertical 09: assemble recipient records

## Evidence from the merged production fixture

CI run 539 for PR 424 completed successfully on head `b6677f12a3d7ad10e678c594446cffdac0dcfffd`.

The public PST fixture now exposes two independently validated row-aligned sequences:

- recipient roles from `PidTagRecipientType`: `to, to, cc, cc`
- recipient display names from `PidTagDisplayName`: `Recipient 1, Recipient 2, Recipient 3, Recipient 4`

The references are all heap IDs and the identity projection completed without a failure reason.

## Milestone

Add a fail-closed assembly boundary that combines recipient role and identity evidence by validated row order.

The milestone produces four structured recipient records in tests:

1. To — Recipient 1
2. To — Recipient 2
3. Cc — Recipient 3
4. Cc — Recipient 4

No record is returned when either evidence sequence is unavailable. A row-count mismatch fails closed and returns no partial records.

## Extraction impact

This converts separate recipient-role and recipient-name arrays into complete role/name records. It does not yet attach SMTP addresses or publish the records from production reporting.

Current public-fixture baseline remains:

- folders discovered: 11
- messages extracted: 1
- body payload records: 2
- recipient rows: 4
- recipient display names: 4
- recipient SMTP addresses: 0
- attachments extracted: 0
- bytes written: 35,332

## Next vertical milestone

Integrate `assemble_recipient_records` into `TcHeapDiagnostic` production reporting and publish the four validated records. After that, select and resolve `PidTagSmtpAddress` (`0x39fe`) or `PidTagEmailAddress` (`0x3003`) from the same rows so each recipient record includes an address.

Do not add another standalone recipient abstraction before production publication.
