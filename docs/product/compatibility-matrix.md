# PSTD Compatibility Matrix

_Last reviewed: 18 July 2026._

## Purpose

Track compatibility by independently testable capability and approved fixture. Passing one PST must not be treated as evidence that the same capability works across the wider PST format.

## Status definitions

| Status | Meaning |
|---|---|
| **Exact** | Fixture output is locked by counts, ownership, byte lengths, hashes, MIME structure, or equivalent deterministic evidence. |
| **Observed** | The capability is present in structured output but is not yet protected by a complete exact regression contract. |
| **Partial** | Some required evidence is extracted, but the object cannot yet be represented completely. |
| **Unsupported** | PSTD identifies the boundary and does not emit a misleading substitute. |
| **Not exercised** | The fixture does not contain suitable evidence for the capability. |
| **Unknown** | The fixture has not yet been assessed for the capability. |

## Approved fixture coverage

| Capability | Original public fixture | Apache Tika `testPST.pst` | Required next evidence |
|---|---|---|---|
| Unicode PST header and root discovery | Exact | Exact | Additional Unicode producers and large files |
| ANSI PST header and root discovery | Not exercised | Not exercised | Pinned public ANSI PST fixture |
| Folder hierarchy | Exact | Exact: eight folder records and seven physical message owners | Additional producers, deleted-item cases, and multi-folder message corpora |
| Message discovery | Exact: 1 message | Exact: 8 messages including one embedded child | Multi-folder and deleted/recovered-message cases |
| Subject and sender metadata | Exact | Exact for validated messages | Broader property combinations and encodings |
| Transport or FILETIME Date | Exact | Exact for emitted parent and child | Conflicting-Date and missing-Date fixtures |
| Message-ID | Exact | Exact where present | Missing and malformed Message-ID cases |
| Direct To recipients | Exact | Exact | Additional Table Context layouts |
| Cc recipients | Exact | Observed | Exact multi-message Cc fixture evidence |
| Bcc recipients | Exact | Not exercised | Public Bcc fixture |
| SMTP addresses | Exact | Exact for six rows | Additional Exchange address combinations |
| Legacy Exchange/native addresses | Not exercised | Exact preservation for native rows | Authoritative SMTP resolution index |
| Plain-text body | Exact | Exact for validated payloads | Additional encodings and large bodies |
| HTML body recovered from RTF | Exact | Unsupported for invalid four-byte child/parent values | Independent valid HTML and RTF fixtures |
| Plain-text-only attachmentless EML | Not exercised | Exact: 453-byte child EML, gated to an attachment-linked embedded message | Additional independently validated plain-only messages |
| Multipart alternative EML | Exact: 956 bytes | Not exercised on current Tika message | Additional valid plain/HTML messages |
| By-value attachment metadata | Not exercised | Exact for one DOCX | More file types and storage layouts |
| By-value attachment payload | Not exercised | Exact: 11,862-byte DOCX with hash and ZIP/CRC evidence | Multiple attachments, large files, inline files |
| Multipart mixed EML | Not exercised | Exact: 17,035-byte parent EML | Multiple attachments and mixed HTML body |
| Method-5 embedded-message link | Not exercised | Exact: one separately owned child with exact standalone EML and payload | Additional producer/layout evidence |
| Embedded-message attachment payload | Not exercised | Exact: 453-byte `message/rfc822`, byte-identical to standalone child EML | Additional layouts and bounded recursion |
| Nested embedded messages | Not exercised | Not exercised | Controlled recursive fixture and depth limit |
| Inline attachments and Content-ID | Not exercised | Not exercised | HTML fixture with verified CID references |
| Contacts | Not exercised | Unknown | Typed contact fixture; never force into EML |
| Distribution lists | Not exercised | Unknown | Typed distribution-list fixture |
| Appointments and recurrence | Not exercised | Unknown | Appointment and recurrence-exception fixtures |
| Tasks, notes, and journals | Not exercised | Unknown | Typed non-mail fixtures as encountered |
| Malformed or truncated PST handling | Partial bounded parser tests | Partial bounded parser tests | Corpus of deterministic corrupt synthetic fixtures |
| Ambiguous ownership rejection | Exact focused tests | Exact for duplicate embedded-child rejection | Additional duplicate and cross-scope cases |
| Deterministic output | Exact | Exact for the 17,035-byte parent and 453-byte child EMLs | Whole-corpus repeatability checks |
| Large-file performance and memory | Unknown | Unknown | Benchmarks with documented resource limits |

## Current release interpretation

PSTD currently demonstrates a material Unicode email extraction path, including recipients, bodies, one by-value DOCX attachment, one attachment-bearing parent EML, and one separately recovered child whose exact EML is also published as a method-`5` `message/rfc822` payload. This is evidence for the approved fixtures only. It is not evidence of general PST compatibility.

No capability may be promoted to broadly supported solely because it passes one fixture. Promotion requires representative fixtures from more than one producer or a controlled structural corpus, exact regression evidence, and explicit malformed-input behaviour.

## Immediate sequence

1. Lock complete folder and message coverage for the Tika fixture.
2. Add independent valid plain-text, HTML, and RTF body fixtures.
3. Add the first pinned public ANSI PST and establish header, tree, folder, message, recipient, and body baselines before extending deeper features.
4. Broaden attachment methods, inline/CID handling, nested messages, and authoritative Exchange-to-SMTP resolution from additional fixtures.
5. Add deterministic corrupt and ambiguous fixture cases before production-readiness claims.

## Maintenance rule

Every extraction pull request must update this matrix when it changes a capability state or adds fixture evidence. The pull request must state the previous status, new status, exact evidence, unsupported boundary, and next unproven case.
