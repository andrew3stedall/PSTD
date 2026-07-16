# Vertical 29: expose the DOCX attachment filename

_Last reviewed: 16 July 2026._

## Objective

Use the pinned Apache Tika `testPST.pst` fixture to expose the first real attachment field from a complete extraction run. The milestone is limited to validated attachment metadata. It does not resolve or emit attachment payload bytes.

## Fixture and owning message

Fixture:

```text
tests/fixtures/upstream/tika-testPST.pst
SHA-256: f2a6b1d2cad00f574e3d1c1211c4b1c854d6526caea77213adc3da92b7813ae3
```

The documented DOCX attachment belongs to:

```text
message_key: msg_c6163b9157944cc9
message_node_id: node_2000e4
subject: FW: First email
```

The source subject currently retains two leading control characters in structured output. Subject cleanup is outside this attachment milestone.

## Evidence boundary

The owning message does not contain `PidTagHasAttachments` in its directly decoded message Property Context. Its recursive subnodes contain three heap Property Contexts:

1. an embedded-message attachment context with method `5` and display name `First email`;
2. the embedded message's own message Property Context;
3. a by-value attachment context containing the DOCX metadata.

The third context provides:

```text
PidTagAttachLongFilename: attachment.docx
PidTagAttachSize:         15503
PidTagAttachMethod:       1
PidTagAttachmentHidden:   false
PidTagAttachDataBinary:   3f830000
```

The four-byte `PidTagAttachDataBinary` value is an unresolved HNID/reference. It is not treated as a four-byte DOCX payload.

## Implementation

The extraction path now:

- considers only recursive subnode blocks that validate as Heap-on-Node Property Contexts with client signature `0xbc`;
- parses their BTH and selected MAPI properties using existing validated components;
- requires a non-empty long or short attachment filename;
- also requires validated attachment method and declared size properties;
- emits one metadata-only `AttachmentRecord`;
- marks the owning message as having one attachment;
- leaves payload bytes unavailable and records that status explicitly;
- rejects non-filename contexts, including the embedded-message attachment, rather than manufacturing a filename.

## Newly readable attachment record

```json
{
  "message_key": "msg_c6163b9157944cc9",
  "filename_original": "attachment.docx",
  "filename_safe": "attachment.docx",
  "extension": "docx",
  "size_bytes": 0,
  "declared_size_bytes": 15503,
  "size_status": "payload_unavailable_declared_size_present",
  "attachment_method": 1,
  "ordinal": 0,
  "extraction_status": "attachment_metadata_extracted_payload_reference_unresolved"
}
```

The owning message now reports:

```text
has_attachments: true
attachment_count: 1
attachment_status: attachment_property_context_filenames_partially_extracted; property_contexts=3; filename_records=1; rejected_contexts=2
extraction_status: metadata_body_and_attachment_metadata
```

## Before versus after

| Measure | Before | After |
|---|---:|---:|
| Messages | 7 | 7 |
| Message JSONL bytes | 23,160 | 21,435 |
| Body records | 8 | 8 |
| Body JSONL bytes | 2,275 | 2,275 |
| Body payload files | 6 | 6 |
| Body payload bytes | 252 | 252 |
| Recipient records | 0 | 0 |
| Recipient JSONL bytes | 0 | 0 |
| Attachment records | 0 | 1 |
| Attachment JSONL bytes | 0 | 605 |
| Attachment payload files | 0 | 0 |
| Attachment payload bytes | 0 | 0 |
| EML files | 0 | 0 |
| EML bytes | 0 | 0 |
| Extraction TAR bytes | 127,488 | 126,464 |
| Total output bytes | 148,716 | 147,692 |

The total output is smaller despite the new 605-byte attachment record because the owning message now carries a concise validated attachment status instead of the much longer generic recursive-subnode diagnostic. TAR block padding accounts for the corresponding archive-size step.

## Verification

The permanent `Tika attachment fixture` workflow asserts:

- the pinned fixture checksum;
- seven messages and eight body records;
- zero recipient records;
- exactly one attachment record;
- the exact filename `attachment.docx`;
- declared size `15,503` and method `1`;
- zero attachment payload files and bytes;
- zero EML files and bytes;
- the exact structured-output and total byte counts above.

Focused unit tests also verify that valid filename, size, and method metadata is emitted and that blank or incomplete contexts fail closed.

## Remaining blockers

- Resolve the HNID carried by `PidTagAttachDataBinary` to the actual attachment bytes.
- Verify that the resolved payload is exactly 15,503 bytes.
- Validate the DOCX ZIP signature and calculate its SHA-256 checksum.
- Recover the embedded-message attachment independently; it is method `5` and must not be flattened into the by-value DOCX path.
- Recipient extraction remains incomplete on this fixture, so no Tika-fixture EML is currently eligible for emission.

## Next vertical milestone

Resolve the `attachment.docx` payload reference and emit exactly one 15,503-byte attachment payload. The milestone must verify the DOCX signature, checksum, owning message, structured record, archive path, and updated output-byte counts before attempting `multipart/mixed` EML assembly.
