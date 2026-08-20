# Upstream source notes

This page makes the register auditable. Links are pinned to the libpst commit used for the comparison.

## `readpst.c`

| Area | Source location | Observation used by this register |
|---|---|---|
| CLI parsing and options | [`main`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/readpst.c#L419-L584) | Flags for charset, deleted items, debug, parallelism, output modes, output types, overwrite, UTF-8 preference, and output directory. |
| Root opening and folder traversal | [`main` and `process`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/readpst.c#L243-L405) | Message store, top-of-folder lookup, recursive children, mixed item types, deleted-folder policy, and typed routing. |
| Output layout creation | [`create_enter_dir`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/readpst.c#L2355-L2448) | Normal, KMail, recursive, separate, Thunderbird, file naming, uniqueness, and per-folder streams. |
| Output layout closing/counts | [`close_enter_dir`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/readpst.c#L2450-L2490) | Empty-file removal, item/skipped/stored counts, and Thunderbird sidecars. |
| Attachment filtering and files | [`acceptable_ext` and `write_separate_attachment`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/readpst.c#L1031-L1112) | Extension allow-list, long/short names, generated names, collisions, and separate payload files. |
| Embedded messages | [`write_embedded_message`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/readpst.c#L1115-L1157) | ID2 resolution, child parsing, `message/rfc822`, and safe skip of non-email children. |
| MIME attachment parts | [`write_inline_attachment`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/readpst.c#L1251-L1308) | Default MIME type, base64, Content-ID, long/short filename handling, RFC 2231, disposition. |
| Header validation and normalization | [`valid_headers` helpers](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/readpst.c#L1310-L1577) | Folded header detection, invalid-body rejection, field extraction, charset/report-type extraction, and stripped duplicates. |
| MIME body writer | [`write_body_part`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/readpst.c#L1579-L1642) | Charset conversion, text transfer encoding, base64 detection, and body-part boundaries. |
| Ordinary email writer | [`write_normal_email`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/readpst.c#L1675-L2004) | Header reconstruction, read status, forensic headers, mixed/alternative/report MIME, RTF/encrypted-body synthetic attachments, schedule parts, and attachment dispatch. |
| Contact output | [`write_vcard`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/readpst.c#L2005-L2170) | vCard field projection, escaping, notes, categories, phones, addresses, and version. |
| Journal and appointment output | [`write_journal` and `write_appointment`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/readpst.c#L2170-L2354) | vJournal/iCalendar fields, recurrence, labels, categories, alarms, and timestamps. |

Line ranges are pointers into the pinned revision; the named functions are the stable reference if upstream formatting changes.

## `libpst.h`

The header defines the requirements that are otherwise easy to miss in `readpst.c`:

- item types and encryption modes near the top of the file;
- attachment method constants and message flags;
- the full email model, including report, RTF, encrypted body, representing-party, and forensic fields;
- contact, journal, recurrence, appointment, and attachment structures;
- file metadata for 32/64-bit families, OST 2013, encryption, indexes, and charset defaults.

See the pinned [header](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/libpst.h) rather than copying only the fields already used by PSTD.

## `libpst.c`

The library source establishes implementation-level requirements for:

- opening and classifying PST/OST files;
- reading encryption state and applying the two supported transformations;
- loading both index trees and extended attributes;
- resolving attachment references and payload chains;
- converting per-property strings to UTF-8;
- deriving item types from message/container classes;
- decoding recurrence data.

The relevant source is [`src/libpst.c`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/libpst.c).

## Regression categories

The upstream regression list includes samples for:

- ANSI and Unicode-era files;
- HTML, plain text, RTF, and embedded images;
- non-ASCII subjects and character sets;
- appointments and recurring appointments;
- MIME-signed email;
- embedded RFC 822 messages;
- journal archives;
- large and unusual archives;
- contacts/distribution-list related data.

PSTD should reuse these categories as fixture requirements without importing private or impermissibly licensed payloads.
