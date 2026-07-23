# PSTD Compatibility Matrix

_Last reviewed: 23 July 2026._

## Purpose

Track compatibility by independently testable capability and approved fixture. Passing one PST must not be treated as evidence that the same capability works across the wider PST format or across Microsoft Purview exports.

## Status definitions

| Status | Meaning |
|---|---|
| **Exact** | Fixture output is locked by counts, ownership, byte lengths, hashes, MIME structure, or equivalent deterministic evidence. |
| **Observed** | The capability is present in structured output but is not yet protected by a complete exact regression contract. |
| **Partial** | Some required evidence is extracted, but the object cannot yet be represented completely. |
| **Unsupported** | PSTD identifies the boundary and does not emit a misleading substitute. |
| **Not exercised** | The fixture does not contain suitable evidence for the capability. |
| **Not admitted** | No approved fixture exists yet for the producer/corpus lane. |
| **Unknown** | The fixture has not yet been assessed for the capability. |

## Approved fixture coverage

| Capability | Original public fixture | Apache Tika `testPST.pst` | Controlled Microsoft Purview corpus | Required next evidence |
|---|---|---|---|---|
| Unicode PST header and root discovery | Exact | Exact | Not admitted | First immutable Purview Unicode export, then additional exports and large files |
| ANSI PST header diagnostics | Not exercised | Not exercised | Not exercised | Deferred; synthetic version-14/15 tests remain exact for diagnostics and fail-closed traversal |
| ANSI PST root traversal and extraction | Unsupported | Unsupported | Not exercised | Backlog only; no active implementation work |
| Folder hierarchy | Exact | Exact: eight folder records and seven physical message owners | Not admitted | Nested Purview folders, deleted-item cases, and multi-folder message corpora |
| Message discovery | Exact: 1 message | Exact: 8 messages including one embedded child | Not admitted | Complete multi-folder and multi-message Purview inventory |
| Subject and sender metadata | Exact | Exact for validated messages | Not admitted | Purview Unicode subjects, producer-specific combinations, and encodings |
| Transport or FILETIME Date | Exact | Exact for emitted parent and child | Not admitted | Purview conflicting-Date and missing-Date cases |
| Message-ID | Exact | Exact where present | Not admitted | Purview present, missing, malformed, and duplicate Message-ID cases |
| Direct To recipients | Exact | Exact | Not admitted | Purview Table Context layouts and completeness counts |
| Cc recipients | Exact | Observed | Not admitted | Exact multi-message Purview Cc evidence |
| Bcc recipients | Exact | Not exercised | Not admitted | Controlled Purview Bcc evidence |
| SMTP addresses | Exact | Exact for six rows | Not admitted | Purview SMTP-native and Exchange combinations |
| Legacy Exchange/native addresses | Not exercised | Exact preservation for native rows | Not admitted | Purview authoritative SMTP mapping evidence and explicit unresolved cases |
| Plain-text body | Exact | Exact for validated payloads | Not admitted | Purview encodings, large bodies, and body-only messages |
| HTML body recovered from RTF | Exact | Unsupported for invalid four-byte child/parent values | Not admitted | Independent valid Purview HTML and RTF evidence |
| Plain-text-only attachmentless EML | Not exercised | Exact: 453-byte child EML, gated to an attachment-linked embedded message | Not admitted | Independently validated ordinary Purview plain-only messages |
| Multipart alternative EML | Exact: 956 bytes | Not exercised on current Tika message | Not admitted | Purview messages with independently valid plain and HTML forms |
| By-value attachment metadata | Not exercised | Exact for one DOCX | Not admitted | Purview messages with zero, one, and multiple attachments |
| By-value attachment payload | Not exercised | Exact: 11,862-byte DOCX with hash and ZIP/CRC evidence | Not admitted | Multiple Purview attachments, formats, non-ASCII filenames, and larger payloads |
| Multipart mixed EML | Not exercised | Exact: 17,035-byte parent EML | Not admitted | Multiple Purview attachments and mixed HTML body |
| Method-5 embedded-message link | Not exercised | Exact: one separately owned child with exact standalone EML and payload | Not admitted | Additional Purview producer/layout evidence |
| Embedded-message attachment payload | Not exercised | Exact: 453-byte `message/rfc822`, byte-identical to standalone child EML | Not admitted | Additional Purview layouts and bounded recursion |
| Nested embedded messages | Not exercised | Not exercised | Not admitted | Controlled Purview recursive fixture and depth limit |
| Inline attachments and Content-ID | Not exercised | Not exercised | Not admitted | Purview HTML with exact `PidTagAttachContentId` and `cid:` correlation |
| Contacts | Not exercised | Unknown | Not admitted | Typed Purview contact fixture; never force into EML |
| Distribution lists | Not exercised | Unknown | Not admitted | Typed Purview distribution-list fixture |
| Appointments and recurrence | Not exercised | Unknown | Not admitted | Purview appointment and recurrence-exception fixtures |
| Tasks, notes, and journals | Not exercised | Unknown | Not admitted | Typed Purview non-mail fixtures as encountered |
| Malformed or truncated PST handling | Partial bounded parser tests | Partial bounded parser tests | Not admitted | Deterministic derivatives of approved Purview synthetic bytes |
| Ambiguous ownership rejection | Exact focused tests | Exact for duplicate embedded-child rejection | Not admitted | Purview duplicate, cross-scope, and conflicting ownership cases |
| Encrypted or unsupported exports | Partial diagnostic boundaries | Partial diagnostic boundaries | Not admitted | Controlled Purview unsupported/encrypted evidence where safely reproducible |
| Deterministic output | Exact | Exact for the 17,035-byte parent and 453-byte child EMLs | Not admitted | Two byte-identical PSTD runs for every admitted Purview fixture |
| Completeness accounting | Partial fixture-specific counts | Exact fixture-specific counts | Not admitted | Exact Purview counts plus explicit unavailable, unsupported, ambiguous, corrupt, and incomplete statuses |
| Large-file performance and memory | Unknown | Unknown | Not admitted | Purview benchmarks with documented parser and resource limits |

## Current release interpretation

PSTD currently demonstrates a material Unicode email extraction path, including recipients, bodies, one by-value DOCX attachment, one attachment-bearing parent EML, and one separately recovered child whose exact EML is also published as a method-`5` `message/rfc822` payload. These fixtures are useful parser and EML evidence, but neither is a controlled Microsoft Purview export.

Microsoft Purview is the primary producer target. Every Purview capability remains **Not admitted** until immutable synthetic export bytes, provenance, redistribution basis, exact hashes, an independent inventory, repeated PSTD output, and explicit completeness statuses are committed. No single `Purview supported` state is permitted.

ANSI support is intentionally parked: only diagnostic header decoding exists, and no ANSI traversal or extraction is claimed.

No capability may be promoted to broadly supported solely because it passes one fixture. Promotion requires representative fixtures from more than one producer or a controlled structural corpus, exact regression evidence, and explicit malformed-input behaviour.

## Immediate sequence

1. Admit the smallest controlled Microsoft Purview Unicode export that exposes a capability not already proven by the existing fixtures.
2. Prefer multiple by-value attachments with exact ownership; otherwise select verified inline CID, authoritative Exchange-to-SMTP mapping, another embedded-message layout, or broader independent HTML/RTF evidence.
3. Lock the complete before-state with exact counts, paths, lengths, hashes, ownership, MIME structure, diagnostics, and two deterministic PSTD runs before changing parser behaviour.
4. Add deterministic corrupt and ambiguous derivatives after the corresponding clean Purview fixture is admitted.
5. Keep ANSI traversal in backlog until Purview Unicode email coverage is materially broader.

## Dependency boundary

PSTD must remain a self-contained Rust parser and EML generator. Do not add java-libpst, libpst, libpff, Apache Tika, Outlook, or another PST parser/converter as a required library, build, runtime, normal test-runtime, CI, Docker, Python-wrapper, or end-user dependency. Pinned external implementations may be used offline or in explicitly isolated fixture-generation and comparison workflows, but acceptance still requires PSTD's own deterministic output against immutable fixture bytes and exact evidence. External-tool agreement is supporting evidence, not sufficient proof.

## Maintenance rule

Every extraction pull request must update this matrix when it changes a capability state or adds fixture evidence. The pull request must state the previous status, new status, exact evidence, unsupported boundary, and next unproven case. Purview rows must remain `Not admitted` until the corresponding controlled fixture passes the full admission gate.
