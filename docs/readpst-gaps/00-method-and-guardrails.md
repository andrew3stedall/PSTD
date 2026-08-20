# Method and parity guardrails

## Purpose

This register answers a narrow question: what observable behaviour must PSTD add or broaden so that a user can replace `readpst` without losing a capability that `readpst` currently provides?

The comparison includes the `readpst` executable and the libpst item/parser surface it uses. It does not silently expand the target to every utility in the repository. `lspst`, `pst2ldif`, `nick2ldif`, and `pst2dii` are recorded only where they expose an item or parser behaviour that readpst also relies on. Their utility-specific output formats are separate future products, not accidental readpst requirements.

## Evidence sources

The inventory was derived from:

| Source | What it establishes |
|---|---|
| [`src/readpst.c`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/readpst.c) | CLI flags, traversal, output modes, MIME generation, attachment handling, vCard, vJournal, and vCalendar output. |
| [`src/libpst.h`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/libpst.h) | Item classes, attachment methods, encryption modes, flags, email/contact/appointment/journal fields, and recurrence structures. |
| [`src/libpst.c`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/libpst.c) | Index loading, decryption, charset conversion, MAPI projection, attachment resolution, item classification, and recurrence decoding. |
| [`NEWS`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/NEWS), [`ChangeLog`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/ChangeLog) | Historical behaviours that are easy to miss when reading only the current command path: OST 2013, Content-ID, mixed item types, RFC 2047/2231, reports, and embedded messages. |
| [`regression/regression-tests.bash`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/regression/regression-tests.bash) | The upstream fixture categories used to exercise HTML, text, appointments, recurrence, embedded messages, non-ASCII headers, MIME-signed mail, and journal archives. |
| PSTD `README.md`, project status, output contract, and approved fixtures | The current PSTD implementation boundary and evidence already available in this repository. |

## What counts as parity

Parity is measured at three layers:

### 1. Input and interpretation

PSTD must open the same relevant PST/OST families, interpret the same encryption and encoding forms, traverse the same folder and item relationships, and resolve the same attachment references. Unsupported or corrupt input may fail closed, but the result must say why and identify the affected scope.

### 2. Information extraction

PSTD must preserve the information readpst uses: folder paths, item classes, message fields, headers, bodies, recipients, attachment metadata and bytes, content IDs, calendar properties, contacts, journals, flags, and recurrence data. A normalized structured record is preferred over losing a field for the sake of matching a legacy output format.

### 3. User-visible outputs

PSTD must offer an equivalent for each readpst output family: mbox, recursive folder output, MH/rfc822, separate files and attachments, KMail, Thunderbird metadata, EML, vCard, vCalendar, vJournal, and MSG. The canonical TAR/JSONL output remains valuable, but it is not by itself proof of output-mode parity.

## Evidence levels

| Level | Use |
|---|---|
| E0 | Source-only observation from upstream code; useful for registering a requirement. |
| E1 | PSTD unit or synthetic structure test; proves a local decoder or writer boundary. |
| E2 | Approved public fixture with deterministic PSTD output; proves one real layout. |
| E3 | Multiple independent producers or a controlled corpus; supports a compatibility claim. |
| E4 | Differential run against readpst plus exact semantic comparison and malformed-input tests; required for final parity claims. |

No row should be marked Implemented for broad compatibility on E0 or E1 alone. The default promotion target is E3; the final parity release requires E4 for every applicable output family.

## Fail-closed requirements

PSTD’s existing correctness rules remain in force:

- do not guess property meaning, address type, encoding, ownership, or row alignment;
- preserve raw values when a stronger interpretation is not justified;
- distinguish absent, empty, unavailable, unsupported, corrupt, and skipped;
- do not combine values from different parser executions into one record;
- bound reads, recursion, allocation, diagnostics, and output filenames;
- retain item and folder counts so skipped content is visible;
- make repeated runs deterministic, including ordering, IDs, hashes, and error statuses.

These rules are a required part of parity. A converter that emits more files by silently inventing values is not a successful replacement for readpst.

## Licensing and implementation boundary

The libpst project is GPL-licensed. PSTD must not add libpst as a required dependency or copy its implementation. The comparison should inform Rust-native code, tests, and fixture design. Any external run used as an oracle must be pinned, isolated, and reported as supporting evidence; PSTD’s own parser and exact output remain authoritative for the project.
