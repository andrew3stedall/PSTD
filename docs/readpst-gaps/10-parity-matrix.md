# readpst parity matrix

This is the single checklist for the gap register. The topic documents explain the behaviours; this matrix is the status and acceptance index.

Status is assessed against PSTD `main` at the time of this review, not against an intended record shape.

## CLI, policy, and operational behaviour

| ID | Capability | Status | Closure evidence |
|---|---|---|---|
| CLI-01 | Help and version | Partial | `pstd` exposes stable help/version plus a documented readpst-parity profile. |
| CLI-02 | Output directory | Implemented | All adapters share a safe output-root contract. |
| CLI-03 | Debug file, level, quiet mode | Partial | Bounded structured logs with severity filtering and deterministic paths. |
| CLI-04 | Parallel jobs | Partial | Bounded deterministic worker pool and identical semantic output across worker counts. |
| CLI-05 | Overwrite/unique output policy | Partial | Explicit policy in run config and adapter tests. |
| CLI-06 | Fallback charset (`-C`) | Gap | Per-run fallback charset with provenance and conversion tests. |
| CLI-07 | Prefer UTF-8 (`-8`) | Partial | Explicit body/output encoding policy with raw-byte retention. |
| CLI-08 | Include deleted items (`-D`) | Gap | Deleted traversal, filter, counts, and scoped records. |
| CLI-09 | Item-type filter (`-t`) | Gap | Email/appointment/journal/contact filters over mixed folders. |
| CLI-10 | Attachment extension allow-list (`-a`) | Gap | Filtered payload status and metadata retention. |
| CLI-11 | Suppress synthetic RTF attachment (`-b`) | Partial | Adapter policy that does not erase canonical RTF evidence. |
| CLI-12 | Contact modes (`-cv`, `-cl`) | Gap | vCard and simple contact list outputs. |

## Input and parser families

| ID | Capability | Status | Closure evidence |
|---|---|---|---|
| IN-01 | Unicode PST traversal | Partial | Multiple producer fixtures with folder/message/body/attachment completeness. |
| IN-02 | ANSI version 14/15 traversal | Gap | Qualifying redistributable ANSI fixture and corrupt derivatives. |
| IN-03 | OST 2013 input | Gap | Qualifying OST fixture and explicit input-contract support. |
| IN-04 | No/compressible/strong encryption | Partial / Gap | End-to-end encrypted fixtures; header classification alone is insufficient. |
| IN-05 | Large-file offsets and streaming | Partial | Sparse/large and real large-file runs without overflow or whole-file loading. |
| IN-06 | BBT/NBT and descriptor traversal breadth | Partial | 32/64-bit page variants, mixed roots, malformed pages, and count reconciliation. |
| IN-07 | Heap/BTH/Property/Table Context breadth | Partial | Selected paths exist; full property and table layout corpus remains. |
| IN-08 | Subnodes and multi-block data | Partial | Direct, nested, XBLOCK/XXBLOCK, and attachment-reference coverage. |
| IN-09 | Extended/named MAPI properties | Partial | Named property mapping and raw preservation across typed items. |
| IN-10 | Corrupt/malicious input safety | Partial | Fuzz/corrupt derivatives with bounded resource use and stable errors. |

## Folders and item classes

| ID | Capability | Status | Closure evidence |
|---|---|---|---|
| ITEM-01 | Folder hierarchy and paths | Partial | Complete folder ownership across producers and duplicate-name cases. |
| ITEM-02 | Content/unread/associated/child counts | Partial | Counts reconcile with typed item statuses. |
| ITEM-03 | Mixed item types in one folder | Gap | One folder fixture containing email, contact, calendar, journal, and skipped classes. |
| ITEM-04 | Deleted and associated contents | Gap | Visible/deleted/associated filtering with explicit counts. |
| ITEM-05 | Ordinary note/email | Partial | Broader message metadata/body/attachment corpus. |
| ITEM-06 | Schedule/meeting email | Gap | Email plus validated `text/calendar` part. |
| ITEM-07 | Appointment/event | Gap | Typed appointment plus iCalendar output. |
| ITEM-08 | Contact | Gap | Typed contact plus vCard/list output. |
| ITEM-09 | Journal | Gap | Typed journal plus vJournal output. |
| ITEM-10 | Sticky note/task/other classification | Gap | Explicit typed metadata and readpst-equivalent skip status. |
| ITEM-11 | Delivery/disposition report | Gap | Typed report and `multipart/report` output. |

## Message fields and body handling

| ID | Capability | Status | Closure evidence |
|---|---|---|---|
| MSG-01 | Sender/representing/received-by identities | Partial | Typed native address fields and SMTP mapping statuses. |
| MSG-02 | To/Cc/Bcc/reply-to row records | Partial | Multiple producer recipient fixtures, including missing and Exchange values. |
| MSG-03 | Dates and FILETIME provenance | Partial | All relevant date fields and deterministic source selection. |
| MSG-04 | Message flags/read state | Gap | Complete flag projection and output-adapter status behaviour. |
| MSG-05 | Importance/priority/sensitivity/receipt controls | Gap | Typed properties and MSG/EML projection tests. |
| MSG-06 | Message-ID/In-Reply-To/References | Partial | References array, duplicate handling, and exact threading status. |
| MSG-07 | Conversation index/topic/normalized subject | Partial | Broader property forms and provenance. |
| MSG-08 | Stored transport header validation | Partial | Folded/invalid/duplicate header corpus and safe normalization. |
| MSG-09 | RFC 2047 header encoding | Gap | Non-ASCII subject/display-name round trips. |
| MSG-10 | RFC 2231 filename encoding | Gap | Non-ASCII and long filename MIME round trips. |
| BODY-01 | Plain text | Partial | Charset and body-only producer coverage. |
| BODY-02 | HTML binary/string forms | Partial | Valid direct HTML, locator, raw, and malformed cases. |
| BODY-03 | Compressed/generic RTF | Partial | Generic RTF payload preservation and decompression corpus. |
| BODY-04 | RTF-to-HTML `fromhtml` | Partial | Broader valid and invalid RTF/HTML fixtures. |
| BODY-05 | Report text | Gap | Report record and multipart/report output. |
| BODY-06 | Encrypted text/HTML preservation | Gap | Synthetic attachment-like output with exact source bytes. |
| BODY-07 | MIME alternative/mixed | Partial | Semantic parser comparison across body/attachment combinations. |
| BODY-08 | Calendar and `message/rfc822` MIME parts | Gap | Schedule and embedded-child MIME trees. |

## Attachments and special objects

| ID | Capability | Status | Closure evidence |
|---|---|---|---|
| ATT-01 | Long/short filename selection | Partial | Unicode, collision, unsafe-name, and missing-name fixtures. |
| ATT-02 | MIME tag/default type | Partial | MIME tag and unknown-type output tests. |
| ATT-03 | By-value method 1 | Partial | Multiple payloads, large/split data, zero length, and size mismatch. |
| ATT-04 | By-reference methods 2/3/4 | Gap | ID2/reference-resolution fixture family. |
| ATT-05 | Embedded message method 5 | Partial | Nested, ambiguous, non-email, and cycle cases. |
| ATT-06 | OLE method 6 | Gap | Lossless OLE bytes and metadata. |
| ATT-07 | Content-ID/inline correlation | Partial | Unique, missing, duplicate, and unmatched CID cases. |
| ATT-08 | Rendering position/MIME sequence | Gap | Order/position retention and MIME projection. |
| ATT-09 | Attachment extension filter | Gap | Case-insensitive filter with metadata/status retention. |
| ATT-10 | Synthetic RTF/encrypted body attachments | Gap | Synthetic-source markers and policy tests. |
| ATT-11 | Nested ownership and recursion limits | Partial | Deterministic parent/child graph and bounded recursion. |

## Output formats

| ID | Capability | Status | Closure evidence |
|---|---|---|---|
| OUT-01 | Canonical structured TAR/JSONL | Implemented | Existing output contract and deterministic fixtures. |
| OUT-02 | Default per-folder mbox | Gap | Semantic mbox reader comparison and mboxrd tests. |
| OUT-03 | Recursive folder mbox | Gap | Folder tree and mixed-type output fixture. |
| OUT-04 | MH/rfc822 separate files | Gap | One-file-per-item output with no mbox separator. |
| OUT-05 | Extended `.eml` output | Partial | Generalized MIME writer and typed item extensions. |
| OUT-06 | Separate binary attachments | Gap | Exact payload files, collisions, filters, and manifest. |
| OUT-07 | KMail layout | Gap | KMail import/read test and safe indexes. |
| OUT-08 | Thunderbird `.type`/`.size` sidecars | Gap | Sidecar count/status tests. |
| OUT-09 | vCard/list | Gap | Contact output round trips. |
| OUT-10 | vCalendar/vJournal | Gap | Calendar/journal parser and recurrence tests. |
| OUT-11 | `.msg` writer | Gap | OLE MSG round-trip and property/recipient/attachment tests. |

## Matrix rule

The overall project cannot claim readpst parity while any applicable row is Gap or Partial. Rows that readpst itself skips still require an explicit PSTD classification and status so the skipped content is visible and auditable.
