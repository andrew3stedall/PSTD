# Vertical 34: Recover the Tika Embedded Message

_Date: 17 July 2026_  
_PR: #455_

## Objective

Recover the method-`5` child in `tests/fixtures/upstream/tika-testPST.pst` as a separate message and link it from the parent attachment without projecting child recipients, bodies, or identifiers onto the parent.

## Root cause

`PidTagAttachDataObject` is PtypObject. Its four-byte Property Context value is HID `0x80`, which points to an exact eight-byte heap allocation:

```text
04 01 20 00  fc 50 00 00
|-- Nid --|  |-- size --|
Nid:       0x00200104
ulSize:    0x000050fc
```

The generic Property Context path previously dereferenced the HID before preserving its object meaning. The recovery path now follows the [MS-PST PtypObject structure](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-pst/49457d57-820e-453d-bbc0-1d192a999814): exactly four NID bytes followed by four size bytes.

The earlier assumption that the attachment PC leaf BID could be matched to an owning SLENTRY was also invalidated by the fixture. The authoritative relationship is the PtypObject NID itself. That NID is admitted only when it has normal-message type `0x04` and resolves exactly once inside the outer message's loaded subnode scope.

## Implementation

- preserve PtypObject HNIDs before generic heap-value dereference;
- parse and validate the exact eight-byte object wrapper;
- reject non-HID object values, zero sizes, wrong NID types, missing allocations, and malformed wrappers;
- require exactly one matching child NID in the parent message scope;
- isolate only the child NID's loaded subnode subtree;
- reuse existing message-property, body, and direct-recipient projection on that isolated scope;
- derive a stable child key from the parent attachment identity and object NID;
- append method-`5` metadata after by-value attachments so the proven DOCX remains ordinal `0`;
- expose `embedded_message_key` on the parent method-`5` record;
- leave nested child attachments and generated method-`5` payload bytes deferred.

## Exact fixture result

| Metric | Vertical 33 | Vertical 34 |
|---|---:|---:|
| Messages | 7 | 8 |
| Body records | 8 | 10 |
| Body payload files / bytes | 6 / 252 | 8 / 279 |
| Recipients | 8 | 9 |
| Attachment records | 1 | 2 |
| Attachment payload files / bytes | 1 / 11,862 | 1 / 11,862 |
| EML files / bytes | 1 / 17,035 | 1 / 17,035 |
| Messages JSONL bytes | 21,521 | 23,086 |
| Bodies JSONL bytes | 2,275 | 2,820 |
| Recipients JSONL bytes | 2,418 | 2,708 |
| Attachments JSONL bytes | 643 | 1,358 |
| TAR bytes | 202,752 | 227,840 |
| Total extraction-output bytes | 241,579 | 272,884 |

## Recovered child evidence

```text
message key:       msg_0ff529af59d373d5
source NID:        0x00200104
data BID:          0x670
subnode BID:       0x67a
recipient:         one To row, raw/native lfcnassif@gmail.com
text body:         23 bytes, "Docx file attached.\r\n\r\n"
raw HTML evidence: 4 bytes, 7f 83 00 00
attachments:       none published; nested extraction deferred
```

The four HTML-property bytes are retained as raw structured evidence. They are not valid markup and do not enter MIME output.

## Parent stability

```text
parent message:    msg_c6163b9157944cc9
direct recipients: unchanged at one
DOCX ordinal/key:  0 / att_0695091e19397627
DOCX bytes:        unchanged at 11,862
outer EML:         unchanged at 17,035 bytes
method-5 ordinal:  1
method-5 key:      att_a9c94a13d70f1cb3
child link:        msg_0ff529af59d373d5
```

The method-`5` record remains metadata-only: `size_bytes=0`, no TAR payload exists at its synthetic `.eml` path, and the empty SHA-256 is preserved. The link, not an empty file, represents the relationship.

## Fail-closed boundary

No child is emitted when the PtypObject value, heap, allocation, object wrapper, normal-message type, NID match, property block, child Property Context, or subtree boundary is missing, malformed, or ambiguous. Duplicate child NIDs inside the parent scope are rejected.

## Validation

The exact fixture workflow verifies all child records and bytes, the complete nine-recipient order, both attachment records, the absence of a method-`5` payload file, the unchanged decoded DOCX and outer MIME tree, and all aggregate byte totals. Rust tests cover object-HID aliasing, exact wrapper shape/type/size, unique child resolution, duplicate rejection, and subtree isolation.

## Next measured boundary

The child has all required headers, received-time Date evidence, one recipient, and a valid UTF-8 plain body. The current attachmentless EML path requires a validated HTML alternative, so it emits no child EML. The next vertical is a deterministic plain-text-only child EML that continues to exclude `7f 83 00 00`. Materialising that EML as the parent method-`5` attachment payload is a later explicit decision.
