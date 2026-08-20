# Parity roadmap and acceptance

The goal is full readpst capability coverage, not a collection of unrelated adapters. Work should proceed in vertical slices that add an observable behaviour and the fixture evidence that proves it.

## Phase 0 — parity contract and typed item envelope

Create the shared foundation before adding many output formats:

- item visibility and item-class classification;
- typed common metadata with source/provenance fields;
- contact, appointment, journal, report, and schedule records;
- attachment method/reference metadata;
- explicit output profile and policy records;
- per-item and per-folder complete/partial/skipped/unavailable counts;
- canonical raw-property and raw-payload retention rules.

Exit criterion: a mixed synthetic folder can be inspected without any item being silently labelled as an ordinary message.

## Phase 1 — Unicode message breadth

Broaden the existing strongest path before adding legacy formats:

- complete sender/representing/received-by and date fields;
- message flags, importance, priority, sensitivity, and report controls;
- valid stored-header normalization and forensic fields;
- String8/Unicode/code-page conversion with `-C` and `-8` policies;
- generic HTML, RTF, report, and body-only cases;
- by-value, inline/CID, multiple, zero-length, size-mismatch, and non-ASCII attachments;
- nested embedded messages with recursion guards.

Exit criterion: a multi-producer Unicode corpus reaches E3 evidence for messages, bodies, headers, recipients, attachments, and embedded children.

## Phase 2 — non-mail and mixed folders

Implement the typed outputs that readpst already exposes:

- vCard and simple contact-list output;
- appointment/event iCalendar output;
- vJournal output;
- schedule/meeting MIME parts;
- delivery/disposition reports;
- mixed-folder routing and `-t` filtering;
- deleted/associated item policy and explicit skip records.

Exit criterion: the pinned `java-libpst-dist-list.pst` and a mixed synthetic fixture produce reconciled typed counts, with contacts, appointments, recurrence, and unsupported classes visible in the structured output.

## Phase 3 — output adapters

Add adapters over validated canonical records:

1. default per-folder mbox and recursive mbox;
2. MH/rfc822 and extended `.eml` separate output;
3. separate binary attachment files and extension filtering;
4. KMail layout;
5. Thunderbird `.type` and `.size` sidecars;
6. contact/calendar/journal file profiles;
7. `.msg` only after its own OLE writer and round-trip gate.

Exit criterion: a semantic differential harness can run the same fixture through readpst and PSTD and compare item counts, folder paths, body/attachment bytes, roles, MIME parts, and typed non-mail values.

## Phase 4 — legacy input and hardening

Add the remaining input breadth:

- ANSI NDB v14/v15 traversal;
- compressible encryption;
- strong encryption;
- OST 2013;
- large-file and sparse-file handling;
- corrupt, truncated, and malicious derivatives;
- bounded parallel execution with deterministic output.

Exit criterion: each supported input family has a real or controlled fixture, exact hashes, independent inventory, repeat-run equality, and no silent loss under malformed derivatives.

## Fixture and differential strategy

Each capability needs a fixture record containing:

```text
source and pinned revision
license/redistribution basis
file size and SHA-256
header family and crypt method
independent folder/item inventory
readpst command and version
PSTD command/profile and version
canonical record counts and hashes
output adapter counts and hashes
known unsupported or ambiguous evidence
```

The differential comparator must compare semantics, not unstable filenames or boundary strings. It should verify:

- folder hierarchy and item class;
- message identity and ownership;
- sender/recipient role and address-type values;
- decoded body bytes and source encodings;
- attachment metadata, method, order, Content-ID, and payload hash;
- calendar/contact/journal fields;
- skipped/unavailable reasons;
- total source versus emitted/partial/skipped counts.

## Definition of done for a row

A parity row can move to Implemented only when:

1. the feature has a Rust-native implementation;
2. the canonical structured output can represent success and failure states;
3. the relevant output adapter exists when readpst exposes one;
4. at least one E2 fixture passes, and broad claims have E3 evidence;
5. a malformed or ambiguous derivative fails closed;
6. output is deterministic across repeated runs and worker counts;
7. the matrix, current-state docs, and changelog are updated;
8. the validation gate and fixture workflow pass.

## Non-negotiable parity properties

- No readpst capability is declared complete solely because a field exists in a Rust struct.
- No output format is implemented by renaming another format.
- No unsupported item is silently counted as a successfully extracted message.
- No native Exchange address is rewritten as SMTP without authoritative evidence.
- No child message or attachment is assigned to a parent from an ambiguous reference.
- No raw body or attachment bytes are discarded because one higher-level projection failed.
- No private PST is committed as a fixture; controlled evidence must meet the repository’s provenance and redistribution rules.
