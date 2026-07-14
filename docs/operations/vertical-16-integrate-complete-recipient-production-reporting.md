# Vertical 16: integrate complete recipient records into production reporting

## Decision

The previous four-row regression removed the remaining shape risk around row ordering and shared-heap resolution. The next highest-value milestone is therefore direct production integration, not another recipient helper or test-only abstraction.

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

`TcHeapDiagnostic` production construction now invokes `project_complete_recipient_records` after the existing fixed-width and single-identity projections. The combined projection receives the existing validated `TcFixedWidthDiagnostic`, so recipient roles are reused rather than decoded a second time.

The bounded combined status fragment is appended to the heap diagnostic status. Existing fixed-width and single-property recipient diagnostics remain unchanged for compatibility and troubleshooting.

No raw row or heap bytes are serialized.

## Expected public fixture evidence

The validated fixture should publish four records from one execution:

```text
0:to:Recipient 1:to1@domain.com:native_email_address
1:to:Recipient 2:to2@domain.com:native_email_address
2:cc:Recipient 3:cc1@domain.com:native_email_address
3:cc:Recipient 4:cc2@domain.com:native_email_address
```

`PidTagEmailAddress` remains classified as `native_email_address`; its values must not be relabelled as SMTP solely because they resemble Internet addresses.

## Acceptance criteria

- formatting, compilation, unit tests, and integration tests pass;
- the public PST fixture completes without regression;
- one production fixture run contains all four complete recipient records;
- incomplete evidence produces no partial complete-record list;
- existing extraction counts and output bytes do not regress unexpectedly.

## Evidence for the following run

Record the exact public-fixture `complete_recipient_records_status`, record list, failure field, message count, body payload count, attachment count, and bytes written.

If all four records are published, recipient-table diagnostics have reached a useful vertical boundary. The following PQ should attach the validated records to the extracted message model and then to EML output, rather than continuing to expand diagnostic formatting.

If production integration fails, use the published display-name, address, role, and complete-record failure evidence to correct the smallest failing boundary. Do not weaken validation or combine rows heuristically.
