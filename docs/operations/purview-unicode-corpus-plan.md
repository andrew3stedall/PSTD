# Microsoft Purview Unicode PST Corpus Plan

_Last reviewed: 23 July 2026._

## Objective

Build representative, evidence-backed compatibility for Unicode PST files exported by Microsoft Purview. Purview is the primary operational producer target; passing unrelated public fixtures is supporting parser evidence but does not establish Purview-export completeness.

## Scope

The corpus must use non-private, redistributable controlled exports. Source mailboxes should be synthetic and contain no production, customer, employee, or personal data. Each PST must retain its original bytes and be pinned by exact byte length and SHA-256.

External implementations, including Outlook, libpff, libpst, java-libpst, and Apache Tika, may be used in isolated fixture-generation or comparison workflows. They are comparison oracles only and must not become PSTD runtime, build, Docker, Python-wrapper, or normal CI dependencies.

## Required fixture families

1. **Small canonical export**
   - multiple folders, including nested folders;
   - multiple ordinary `IPM.Note` messages;
   - To, Cc, and Bcc recipients;
   - plain-text, HTML, and RTF-backed bodies;
   - deterministic dates, Message-IDs, and Unicode subjects.

2. **Attachment export**
   - zero, one, and multiple by-value attachments on separate messages;
   - several file formats and non-ASCII filenames;
   - at least one larger payload;
   - exact ownership, byte length, SHA-256, MIME type, and EML placement.

3. **Inline-content export**
   - HTML with one and multiple `cid:` references;
   - matching `PidTagAttachContentId` values;
   - inline and ordinary attachments on the same message;
   - no inferred Content-ID when source evidence is absent or ambiguous.

4. **Exchange-address export**
   - SMTP-native recipients;
   - legacy Exchange distinguished names;
   - authoritative SMTP mapping evidence where Purview exports it;
   - explicit unresolved status where no authoritative mapping exists.

5. **Embedded-message export**
   - one method-5 child;
   - multiple embedded children;
   - bounded nested embedded messages;
   - duplicate, cross-scope, and recursion-limit rejection cases.

6. **Non-mail export**
   - contacts, distribution lists, appointments, tasks, notes, and journals where Purview includes them;
   - typed classification and completeness status;
   - no forced EML projection.

7. **Malformed and boundary derivatives**
   - controlled truncation and corruption derived from approved synthetic bytes;
   - encrypted or unsupported cases where safely reproducible;
   - exact fail-closed diagnostics and zero guessed output.

## Admission record

Every fixture requires:

- Purview export procedure and relevant export settings;
- source synthetic-mailbox manifest;
- declared redistribution basis;
- original filename, byte length, and SHA-256;
- PST header version and encryption classification;
- independent object inventory from at least one pinned comparison implementation;
- PSTD baseline output captured twice to prove determinism;
- exact expected folder, message, recipient, body, attachment, typed-object, diagnostic, and EML counts;
- exact payload and EML paths, lengths, hashes, and MIME structure where materialised;
- explicit unavailable, unsupported, ambiguous, corrupt, or incomplete records.

## First vertical acceptance

The first admitted Purview fixture should be the smallest controlled Unicode export that exposes a capability not already proven by the current fixtures. Preference order:

1. multiple by-value attachments with exact ownership;
2. inline attachment with exact HTML `cid:` correlation;
3. authoritative Exchange-to-SMTP mapping;
4. another embedded-message layout or bounded recursion;
5. broader independent HTML/RTF body evidence.

Before implementation, lock PSTD's current deterministic baseline. After implementation, require one clear before/after result and preserve all existing fixture identifiers and bytes unless an intentional contract change is documented.

## Fail-closed boundary

PSTD must not infer message class, ownership, recipient mapping, body form, inline disposition, Content-ID, attachment payload, or embedded-message linkage from names, folder placement, ordering, or another parser's interpretation. Missing or conflicting evidence remains explicit and cannot silently disappear.

## Compatibility reporting

The compatibility matrix should report Purview coverage by fixture and capability rather than using a single `Purview supported` claim. PSTD must not be described as generally reliable for Purview exports until representative fixture families pass with exact completeness statuses and no silent data loss.