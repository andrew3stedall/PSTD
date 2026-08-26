# Storage and interoperability outputs

PSTD’s TAR/JSONL archive is the correct canonical evidence boundary for the project. It is not, however, a substitute for every output format that readpst offers. The compatibility layer should consume validated records and raw artefacts without reparsing the PST.

## Mailbox and message-file formats

| Format | readpst result | PSTD status | Required behaviour |
|---|---|---|---|
| mbox | One file per folder/type, multiple messages with mbox `From ` separators. | **Partial** | `src/output/mailbox.rs` emits deterministic mboxrd streams over canonical mail records; reduced-type and broad-corpus evidence remain. |
| recursive mbox (`-r`) | PST folder tree as directories; each folder has a reduced-type mailbox. | **Partial** | `recursive_mbox` recreates sanitized folder paths and records skipped/unavailable decisions; typed side streams remain. |
| MH/rfc822 (`-M`) | One message per numbered file, no separator line. | **Partial** | `mh` emits folder-local numbered files with no mbox separator. |
| separate with extensions (`-e`) | Numbered `.eml`, `.vcf`, and `.ics` files. | **Partial** | `eml` emits numbered `.eml`; contact/calendar/journal extensions are covered by their named profiles and broader combination tests remain. |
| separate with MSG (`-m`) | Extended separate output plus `.msg`. | **Partial** | Rust-native CFB/OLE MSG plus deterministic `.eml` companion, supported-property map, and independent reader gate; named properties and embedded method-5 breadth remain explicit. |
| KMail (`-k`) | `.folder.directory` layout and mbox files suitable for KMail. | **Partial** | `kmail` emits safe `.<folder>.directory/<folder>.mbox` entries and an explicit index-invalidation policy; import/read compatibility remains. |
| Thunderbird (`-u`) | Recursive output plus `.type` and `.size` files per folder. | **Partial** | `thunderbird` emits recursive mbox, canonical-identity `.type`/`.size` sidecars, and independent typed non-mail files; exact import compatibility remains. |

## EML and MIME compatibility

PSTD already produces deterministic EML for selected plain/HTML, by-value attachment, and one embedded-message cases. To reach readpst parity, the writer must additionally handle:

- stored valid headers plus reconstructed missing headers;
- mbox separators only where the selected output format needs them;
- RFC 2047 encoded generated subjects/display names;
- RFC 2231 filename parameters and safe fallback names;
- report, schedule, and embedded-message MIME types;
- Content-ID and inline disposition;
- encrypted and synthetic RTF artefacts;
- attachment methods and unavailable payload statuses;
- stable MIME boundary generation and nested ownership.

The EML adapter should expose a semantic validation report: decoded headers, part tree, body hashes, attachment hashes, and warnings. A byte comparison to readpst is useful but not sufficient because readpst’s boundary strings and formatting are implementation details.

The current output expansion supplies deterministic RFC 2047/RFC 2231 generation and
cross-profile extension filtering for mailbox and MSG/EML projections. Stored-header
normalization, full MIME reader round trips, and broad readpst semantic differentials
remain Partial.

## Contact and calendar files

The output extension and stream mapping are part of compatibility:

```text
contact vCard       -> .vcf (or contact mbox/list profile)
appointment/event   -> .ics / text/calendar
journal             -> .ics or a documented vJournal file profile
email               -> .eml / mbox / MH / MSG profile
```

The adapter must not place different item classes in one file merely because they share a folder.

RP-M4-02 adds the first `icalendar` profile over canonical `CalendarRecord` values.
RP-M4-03 adds a Partial `vjournal` profile over canonical `NonMailRecord` journal
values. These adapters write deterministic `outputs/appointments.ics` or
`outputs/journals.vjournal` projections plus status records. Missing appointment or
journal property groups remain explicit; neither adapter maps non-mail items into
ordinary email.

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

An EML file with a `.msg` extension is not an acceptable substitute. PSTD now emits
a real CFB/OLE container through `src/output/msg.rs`; the compatibility EML is a
separate artifact and never stands in for the MSG file.

## Structured output remains authoritative

Legacy adapters must be projections. They must never change the canonical counts or suppress source metadata. A run that emits no EML because a required header/body is unavailable can still be a successful structured extraction with `eml_status=unavailable`; it must not be reported as a complete readpst-equivalent export.

## Planned implementation — `RP-09`

### Readpst logic reviewed

`create_enter_dir` maps appointment/contact/journal to separate streams and reduces other types to the note/mbox stream; it creates unique names from `item->file_as`. `mk_recurse_dir` and `mk_separate_dir` build folder trees; `mk_separate_file` numbers items from one per folder and chooses `.eml`, `.vcf`, or `.ics`; `close_enter_dir` removes empty streams and reports counts. `mk_kmail_dir` writes `.folder.directory` and removes the parent KMail index. `write_normal_email` supplies mbox separators, MH differences, MIME output, separate attachments, and `.msg` dispatch. `msg.cpp::write_msg_email` creates an OLE compound document with `__properties_version1.0`, top-level MAPI properties, recipient storages, attachment storages, and `__nameid_version1.0`; it writes strings as ANSI properties after a hard-coded transliteration charset, emits selected flags/dates/body/header properties, and explicitly does not implement embedded attachments. The regression script exercises recursive, contact, charset/filter, separate, and MSG combinations.

### Planned adapter architecture

Define a common adapter trait over canonical records:

```text
OutputAdapter::begin(run, folder) -> AdapterContext
OutputAdapter::write(item_envelope, evidence_graph) -> AdapterResult
OutputAdapter::finish(folder) -> FolderOutputSummary
```

Implement adapters in dependency order: `mbox`, `recursive_mbox`, `mh`, `eml`, `separate_attachments`, `kmail`, `thunderbird`, `vcard/list`, `icalendar/vjournal`, then `msg`. Reuse `src/output/paths.rs`, JSONL/TAR IDs, metadata records, MIME serialization, and atomic write helpers. Every adapter writes a manifest entry with source key, path, hash, status, and warnings.

### Implementation flow

1. Construct the adapter from `RP-01`’s profile and validate that every selected item kind has a compatible projection.
2. Walk canonical folder/item order; never re-open or reparse the PST. Use a folder-local ordinal for readpst-compatible separate filenames and a source-ID component for collision safety.
3. For mbox, emit one stream per folder/reduced type, mboxrd-escape body lines beginning with `From `, and record message offsets/hashes. For MH/EML, omit separators and write one complete message per file.
4. For recursive/KMail/Thunderbird, map canonical folder segments to safe paths and emit sidecars/counts from the same `FolderOutputSummary`.
5. For vCard/list/calendar/journal, dispatch typed records and choose `.vcf`/`.ics`/documented vJournal extensions without placing different classes in an email stream.
6. For separate attachment profiles, emit payload files only when their `AttachmentRecord` status is available and not filtered; record all other decisions.
7. Implement MSG in a dedicated Rust OLE compound-document module. Generate a property table with explicit MAPI types, top-level flags/dates/subject/body/header fields, recipient rows, attachment storages, and NameID entries. Do not use an EML body with a `.msg` suffix. The `msg` profile now implements this bounded map, records unsupported values, and emits a separate `.eml` companion for the `-m` surface.
8. Round-trip every output through a semantic reader before marking the adapter result complete. Atomic publish happens only after validation.

### `.msg` implementation boundary

`msg.cpp` is a useful compatibility map, not a complete MSG implementation. The first MSG issue must define the supported property matrix and OLE writer contract. It must cover Unicode and ANSI strings, FILETIME, boolean/integer flags, recipients with To/Cc/Bcc types, by-value attachments, MIME tags, long/short names, record keys, body/HTML/header/message IDs, and deterministic stream names. Embedded messages, named properties, OLE attachments, and full recipient-row semantics require separate issues or explicit `unsupported` statuses. A trusted MSG parser must verify the resulting compound file.

### Improvements over readpst

- Keep adapters pure projections over canonical evidence and remove `chdir`/global stream state.
- Preserve empty, filtered, skipped, unavailable, and failed results rather than deleting empty files without explanation.
- Use stable source IDs plus sanitized paths, atomic publication, and collision policies that are explicit across platforms.
- Generate MIME/ICS/vCard/MSG with standards-aware serializers and independent round-trip validation.
- Keep a stronger canonical record even when a legacy profile is intentionally lossy.
- Make `.msg` capability honest: partial property coverage is a scoped status, never a mislabeled text file.
- Validate every archive path as relative and confined, and publish TAR shards through a
  close-then-rename `.part` boundary so failed runs cannot expose a final incomplete
  archive.

### Issue-ready acceptance

`RP-09A` is mbox/recursive, `RP-09B` MH/EML/separate, `RP-09C` KMail/Thunderbird, `RP-09D` contact/calendar/journal file profiles, and `RP-09E` MSG/OLE. For every profile, compare folder/item counts, decoded fields, body/attachment hashes, typed output, paths, and statuses; test reruns, collisions, overwrite modes, malformed evidence, worker counts, and path traversal. MSG additionally requires an OLE reader round trip and property/recipient/attachment matrix. Update [CLI policy](01-cli-and-output-parity.md), [metadata](04-message-metadata-and-headers.md), [bodies](05-body-mime-and-rtf.md), [attachments](06-attachments.md), [non-mail outputs](08-contacts-calendar-journal.md), and the matrix.

RP-M3-03 establishes the storage-facing policy contract before adapter work: output
roots remain explicit, collision/overwrite choices are serialized, job counts are
bounded, and repeated canonical runs preserve identical policy and item JSONL. Named
legacy profiles are parsed for workboard traceability but produce an explicit
unsupported status rather than writing a misleading canonical or adapter output.

RP-M4-01 adds `vcard` and `contact_list` as the first implemented named output
profiles. They consume `ContactRecord` values from canonical extraction, publish
deterministic `contacts.vcf`/`contacts.txt` projections, and emit an explicit
profile-status record when the source contains no validated contacts. No contact
profile reparses PST bytes or promotes missing contact fields.

RP-M5-01 adds the first mailbox adapter family. The implementation is a pure projection
over `MessageRecord`, `HeaderProjectionRecord`, body payloads, recipients, and available
attachment payloads; it never reparses PST bytes. It publishes profile status, per-message
decisions, output hashes, and `data/mailbox_adapter_manifest.jsonl` inside the canonical
TAR. Paths are sanitized relative paths with stable folder collision suffixes. Complete
serialization is performed before archive publication, so a failed or unavailable message
does not leave a partial direct output file. This slice is Partial until the pinned
readpst differential corpus and downstream attachment/KMail/Thunderbird gates pass.
The attachment/KMail portion of that downstream boundary is now covered by RP-M5-02;
the remaining promotion work is differential and broad-corpus validation.

RP-M5-02 extends the same projection with separate binary attachment files and KMail.
The separate profile follows readpst’s `<message-file>-<filename>` shape, prefers the
canonical safe filename, applies the normalized `-a` allow-list, and records filtered,
unavailable, unsafe, and integrity-failed decisions while publishing every validated
payload, including empty and recovered embedded-message bytes. KMail emits
`.<folder>.directory/<folder>.mbox` paths and records that
the mutable parent index is logically invalidated; it does not emit an index file. The
attachment/KMail workflow proves repeated output equality and archive path safety.

RP-M5-03 adds Thunderbird sidecars and typed non-mail files over canonical records.
RP-M5-04 adds a Rust-native CFB/OLE MSG projection with UTF-16 property streams,
recipient/attachment storages, deterministic output, and an independent `olefile`
round-trip gate. Method-5 embedded attachments and unrepresentable properties remain
explicit status decisions; raw canonical evidence remains authoritative.
`.size` retains the readpst two-count form; `.type` is a deterministic JSON
stronger-equivalent with an explicit null type when the canonical folder schema does not
expose the numeric source type. Contacts, appointments, journals, and preserved
unsupported non-mail records are serialized independently of ordinary email, with source
identity and unsupported-field decisions retained.
# RP-M2-03 delivery

Canonical attachment output now records method/source, order, CID, original and safe
names, payload hash, size status, and adapter-independent extraction status. Archive
paths continue to be sanitized and bounded before TAR publication; the attachment
source fields make file/MIME/MSG adapters consume the same evidence graph.

## RP-M2-04 canonical storage

The archive now includes `data/mime_parts.jsonl` beside bodies, attachments, headers,
and evidence. MIME boundaries are represented as stable part ownership keys derived
from the message key and semantic role; adapter serializers can choose their own
wire boundary while consuming the same raw/decoded hashes and explicit statuses.
