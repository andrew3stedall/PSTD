# Upstream PST fixture corpus

_Last reviewed: 20 July 2026._

## Purpose

PSTD's original public fixture is deliberately small and has been useful for validating one readable message, four recipients, plain text, RTF, recovered HTML, and deterministic EML assembly. It contains no attachment candidates and does not provide enough variation to validate broader PST compatibility.

This corpus adds three public, pinned PST files so extraction can advance through observable vertical milestones without using private client data. Each file has a different role. Do not treat the corpus as proof of general PST compatibility.

## Fixture inventory

| Local file | Size | SHA-256 | Primary purpose |
|---|---:|---|---|
| `tests/fixtures/upstream/tika-testPST.pst` | 2,302,976 bytes | `f2a6b1d2cad00f574e3d1c1211c4b1c854d6526caea77213adc3da92b7813ae3` | Multiple messages and folders, Unicode metadata, a documented DOCX attachment, and a legacy Exchange recipient address |
| `tests/fixtures/upstream/tika-various-body-types.pst` | 271,360 bytes | `24c5e6bbb8bf26a817c977283e40e7b69d2661fec0845abbe177f97efcb05fb0` | Multiple body representations and body-selection behaviour |
| `tests/fixtures/upstream/java-libpst-dist-list.pst` | 271,360 bytes | `c86841da106036b5abe5a2141dc7644cbb2bf8b504873515eb35a2efeb8c28ac` | Recurring appointments, recurrence exceptions, contacts, distribution lists, and non-mail MAPI objects |

Machine-readable hashes and sizes are stored beside the fixtures in `SHA256SUMS` and `FILE-SIZES.tsv`.

## Provenance

### Apache Tika `testPST.pst`

- Upstream repository: `apache/tika`
- Pinned commit: `63e22d08ef249cc73a6d02da7bc199fc3623a607`
- Upstream path: `tika-app/src/test/resources/test-data/testPST.pst`
- Source URL: `https://github.com/apache/tika/blob/63e22d08ef249cc73a6d02da7bc199fc3623a607/tika-app/src/test/resources/test-data/testPST.pst`
- Upstream project licence: Apache License 2.0

Apache Tika's regression tests document the following expected evidence:

- a 2,302,976-byte PST;
- multiple email-folder elements, including a non-ASCII French folder name;
- ten recursive metadata objects including the PST container;
- messages with Unicode names, including `Jörn Kottmann`;
- a nested `attachment.docx` whose content includes `This is a docx attachment.`;
- a legacy Exchange distinguished-name recipient beginning `/o=ExchangeLabs/ou=Exchange Administrative Group ...`.

The known attachment path reported by Tika is:

```text
/ First email.msg/First email.msg/attachment.docx
```

### Current PSTD evidence from `testPST.pst`

PSTD currently emits seven top-level messages plus one separately linked method-`5` child, ten body records, nine directly owned recipient records, and two attachment records. The known DOCX-bearing outer message is:

```text
message_key: msg_c6163b9157944cc9
message_node_id: node_2000e4
subject: FW: First email
```

Its direct message Property Context omits `PidTagHasAttachments`, but its recursive subnodes contain a validated filename-bearing attachment Property Context with:

```text
PidTagAttachLongFilename: attachment.docx
PidTagAttachSize:         15503
PidTagAttachMethod:       1
PidTagAttachmentHidden:   false
PidTagAttachDataBinary:   3f830000
```

The four-byte data value resolves through a complete Unicode SLBLOCK to the internal data-tree BID:

```text
data NID:      0x0000833f
resolved BID:  0x632
```

BID `0x632` is a Unicode XBLOCK with two ordered external child BIDs. Its `lcbTotal` is 11,862 bytes, which differs from the 15,503-byte `PidTagAttachSize` metadata value. PSTD preserves both values and emits the XBLOCK payload exactly:

```text
payload files/bytes: 2/12315
SHA-256:             0c87a742c970907d3b08c73e7834768abadd00fe4f4995a7dd98a206d4c494c0
ZIP signature:       50 4b 03 04
DOCX CRC validation: passed
expected text:       This is a docx attachment.
```

The payload is written to:

```text
attachments/msg_c6163b9157944cc9/att_0695091e19397627_attachment.docx
```

Filename evidence is recorded in [Vertical 29](vertical-29-expose-docx-attachment-filename.md), exact reference-resolution evidence in [Vertical 30](vertical-30-resolve-docx-attachment-data-reference.md), and DOCX payload evidence in [Vertical 31](vertical-31-emit-docx-attachment-payload.md). Recipient and outer-EML evidence is recorded in Verticals 32-33, embedded-message recovery in [Vertical 34](vertical-34-recover-tika-embedded-message.md), exact child EML evidence in [Vertical 35](vertical-35-emit-tika-child-eml.md), method-`5` payload evidence in [Vertical 36](vertical-36-materialise-method5-eml-payload.md), complete folder/message ownership in [Vertical 37](vertical-37-resolve-tika-message-folder-ownership.md), and fail-closed independent body-form selection in [Vertical 38](vertical-38-reject-unresolved-binary-body-references.md).

The fixture now produces two deterministic EML files: the unchanged 17,035-byte parent with the exact DOCX and a 453-byte single-part plain-text child. The child EML is also published byte-for-byte as the method-`5` `message/rfc822` attachment payload at the stable attachment path. Attachment payload output is 2 files / 12,315 bytes. All eight folder records and all seven top-level physical owners are exact. Two four-byte Property Context HTML locators are retained as explicit unavailable forms rather than emitted as payloads.

### Apache Tika `testPST_variousBodyTypes.pst`

- Upstream repository: `apache/tika`
- Pinned commit: `63e22d08ef249cc73a6d02da7bc199fc3623a607`
- Upstream path: `tika-parsers/tika-parsers-standard/tika-parsers-standard-modules/tika-parser-microsoft-module/src/test/resources/test-documents/testPST_variousBodyTypes.pst`
- Source URL: `https://github.com/apache/tika/blob/63e22d08ef249cc73a6d02da7bc199fc3623a607/tika-parsers/tika-parsers-standard/tika-parsers-standard-modules/tika-parser-microsoft-module/src/test/resources/test-documents/testPST_variousBodyTypes.pst`
- Upstream project licence: Apache License 2.0

Apache Tika's regression test expects five recursive metadata objects and uses the fixture specifically to exercise PST messages with different body forms. PSTD now locks the valid 37-byte plain body while retaining the four-byte HTML locator as an explicit unavailable form.

### java-libpst `dist-list.pst`

- Upstream repository: `rjohnsondev/java-libpst`
- Pinned commit: `f158a64acf2a0e46ac3bd699bc7a5a8da6c40d26`
- Upstream path: `src/test/resources/dist-list.pst`
- Source URL: `https://github.com/rjohnsondev/java-libpst/blob/f158a64acf2a0e46ac3bd699bc7a5a8da6c40d26/src/test/resources/dist-list.pst`
- Upstream repository publishes Apache 2.0 and LGPL licence texts. Preserve this provenance if the fixture is redistributed.

java-libpst's tests document:

- a recurring appointment object;
- three deleted occurrence dates;
- two modified recurrence exceptions;
- exception descriptions `This is the appointment at 9` and `This is the one at 10`;
- a distribution list with three members;
- one stored contact and two one-off recipients.

This fixture should be introduced only after ordinary mail-message extraction is stable across the Tika samples. Appointment and distribution-list objects must not be forced through the normal email path.

## ANSI fixture qualification

The first ANSI baseline must be proven from the PST header before it is approved. The accepted file must have:

- `!BDN` at bytes 0-3;
- PST client magic `SM` at bytes 8-9;
- NDB version 14 or 15 at bytes 10-11;
- a public, redistributable source with a pinned commit or immutable release;
- exact size and SHA-256 recorded before parser behaviour is measured.

The libyal public candidate `libyal/testdata:pst/outlook.pst` was inspected on 20 July 2026 and rejected for the ANSI milestone. Its bytes 10-11 are `17 00`, declaring NDB version 23 (Unicode). The filename, project age, and presence of ANSI string support are not evidence that a PST uses the 32-bit ANSI format. This file may later broaden Unicode producer coverage, but it must not be represented as ANSI evidence.

No extraction code should be changed until a qualifying version-14 or version-15 fixture is pinned. If no redistributable file can be found, the next safe step is a controlled synthetic ANSI structure with documented generation provenance, not relabelling a Unicode sample.

## Address-type boundary

The address in `tika-testPST.pst` beginning `/o=ExchangeLabs/...` is a legacy Exchange `EX` address or distinguished name. It is X.500-style evidence, not a true X.400 O/R address.

Use it to validate this bounded behaviour:

```text
legacy Exchange address
    -> preserve the raw address and address type
    -> search only validated same-PST evidence for an SMTP mapping
    -> emit SMTP when uniquely resolved
    -> otherwise retain the raw address and mark it unresolved
```
