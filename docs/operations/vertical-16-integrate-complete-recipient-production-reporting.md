# Vertical 16: integrate complete recipient records into production reporting

## Decision

The previous four-row regression removed the remaining shape risk around row ordering and shared-heap resolution. The next highest-value milestone was direct production integration, not another recipient helper or test-only abstraction.

## Requirement

For every resolved subnode-backed Table Context heap, production reporting must reuse the same validated row payload, bitmap masks, column descriptors, heap bytes, and `PidTagRecipientType` evidence to project both:

- `PidTagDisplayName`;
- the preferred available address property (`PidTagSmtpAddress`, otherwise `PidTagEmailAddress`).

It must then publish complete row-aligned records containing:

```text
row_index + role + display_name + address + address_kind
```

Any missing descriptor, bitmap mismatch, unresolved HNID, malformed string, semantic mismatch, or row-count mismatch must fail closed and publish no partial complete-record list.

## Implementation

`TcHeapDiagnostic` production construction invokes `project_complete_recipient_records` after the existing fixed-width and single-identity projections. The combined projection receives the existing validated `TcFixedWidthDiagnostic`, so recipient roles are reused rather than decoded a second time.

The bounded combined status fragment is appended to the heap diagnostic status. Existing fixed-width and single-property recipient diagnostics remain unchanged for compatibility and troubleshooting. No raw row or heap bytes are serialized.

## Public fixture evidence

GitHub Actions run 565 executed `tests/fixtures/pst/sample.pst` and published:

```text
complete_recipient_records_status=tc_recipient_records_validated
complete_recipient_records=0:to:Recipient 1:to1@domain.com:native_email_address|1:to:Recipient 2:to2@domain.com:native_email_address|2:cc:Recipient 3:cc1@domain.com:native_email_address|3:cc:Recipient 4:cc2@domain.com:native_email_address
complete_recipient_records_failure=none
```

`PidTagEmailAddress` remains classified as `native_email_address`; its values are not relabelled as SMTP solely because they resemble Internet addresses.

This affects one extracted message and exposes four complete recipient records in one production fixture execution.

## Before versus after

| Measure | Before | After |
|---|---:|---:|
| Messages discovered | 1 | 1 |
| Messages extracted | 1 | 1 |
| Body payload records | 2 | 2 |
| Complete recipient records in production output | 0 | 4 |
| Attachments extracted | 0 | 0 |
| EML files emitted | 0 | 0 |
| Output bytes | 35,371 | 39,622 |

The 4,251-byte increase is bounded diagnostic and structured extraction evidence, not message-body, attachment, or EML output.

## Verification

- GitHub Actions run 565 completed successfully on head `344f58f9b936a084ac20e726f5b92c78987a6450`.
- The public fixture artifact reported one message, two body payload records, zero attachments, zero EML files, and 39,622 output bytes.
- All four recipient rows retained role, display name, address, and address kind.
- The combined projection reported no failure and emitted no partial records.

## Next vertical milestone

Attach these four validated records to the extracted message model and use them as the source for structured `To` and `Cc` output. Do not add another diagnostic, wrapper, or formatter-only layer. The next milestone must make the recipients available on the message record and, if the existing serializer boundary permits within the same coherent slice, emit corresponding EML headers.