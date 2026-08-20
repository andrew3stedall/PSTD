# Storage and interoperability outputs

PSTD’s TAR/JSONL archive is the correct canonical evidence boundary for the project. It is not, however, a substitute for every output format that readpst offers. The compatibility layer should consume validated records and raw artefacts without reparsing the PST.

## Mailbox and message-file formats

| Format | readpst result | PSTD status | Required behaviour |
|---|---|---|---|
| mbox | One file per folder/type, multiple messages with mbox `From ` separators. | **Gap** | Emit deterministic mbox with mboxrd escaping and preserved message order. |
| recursive mbox (`-r`) | PST folder tree as directories; each folder has a reduced-type mailbox. | **Gap** | Recreate folder hierarchy safely and keep each item type in its own stream. |
| MH/rfc822 (`-M`) | One message per numbered file, no separator line. | **Gap** | Emit individual RFC 822/EML files and sidecar attachments according to policy. |
| separate with extensions (`-e`) | Numbered `.eml`, `.vcf`, and `.ics` files. | **Partial** | Generalize current EML assembly and add typed non-mail extensions. |
| separate with MSG (`-m`) | Extended separate output plus `.msg`. | **Gap** | Provide a tested MSG writer or clearly scoped equivalent; do not generate a mislabeled EML. |
| KMail (`-k`) | `.folder.directory` layout and mbox files suitable for KMail. | **Gap** | Add a KMail adapter with safe folder names and documented index behaviour. |
| Thunderbird (`-u`) | Recursive output plus `.type` and `.size` files per folder. | **Gap** | Emit the two sidecars from canonical counts and preserve skipped/unavailable counts separately. |

## EML and MIME compatibility

PSTD already produces deterministic EML for selected plain/HTML, by-value attachment, and one embedded-message cases. To reach readpst parity, the writer must additionally handle:

- stored valid headers plus reconstructed missing headers;
- mbox separators only where the selected output format needs them;
- RFC 2047 encoded subjects/display names;
- RFC 2231 filename parameters and safe fallback names;
- report, schedule, and embedded-message MIME types;
- Content-ID and inline disposition;
- encrypted and synthetic RTF artefacts;
- attachment methods and unavailable payload statuses;
- stable MIME boundary generation and nested ownership.

The EML adapter should expose a semantic validation report: decoded headers, part tree, body hashes, attachment hashes, and warnings. A byte comparison to readpst is useful but not sufficient because readpst’s boundary strings and formatting are implementation details.

## Contact and calendar files

The output extension and stream mapping are part of compatibility:

```text
contact vCard       -> .vcf (or contact mbox/list profile)
appointment/event   -> .ics / text/calendar
journal             -> .ics or a documented vJournal file profile
email               -> .eml / mbox / MH / MSG profile
```

The adapter must not place different item classes in one file merely because they share a folder.

## Path and collision semantics

All adapters need common path policy:

- use canonical folder names for records and sanitized names only for paths;
- prevent traversal and reserved-name collisions;
- preserve duplicate folders/items with stable source-derived suffixes;
- make overwrite and skip policies explicit;
- retain the original attachment filename alongside the safe path filename;
- avoid silently replacing an existing file;
- write through a temporary file and atomically publish complete outputs where practical;
- record generated paths and hashes in the manifest.

## MSG boundary

readpst’s `-m` mode writes Microsoft OLE MSG files through `msg.cpp`, including a top-level property stream, string/binary properties, recipient streams, attachment streams, and selected email properties. This is a separate writer with a large fidelity surface. PSTD should not claim MSG parity until it has:

1. a Rust-native writer or a clearly isolated writer implementation;
2. round-trip tests against a trusted MSG reader;
3. Unicode and ANSI property tests;
4. recipients, attachments, embedded messages, dates, and named-property handling;
5. deterministic output and safe failure on unsupported properties.

An EML file with a `.msg` extension is not an acceptable substitute.

## Structured output remains authoritative

Legacy adapters must be projections. They must never change the canonical counts or suppress source metadata. A run that emits no EML because a required header/body is unavailable can still be a successful structured extraction with `eml_status=unavailable`; it must not be reported as a complete readpst-equivalent export.
