# readpst parity matrix

This is the single checklist for the gap register. The topic documents explain the behaviours; this matrix is the status and acceptance index.

Status is assessed against PSTD `main` at the time of this review, not against an intended record shape.

## RP-M1-04 delivery

The typed-core provenance slice is implemented on the canonical extraction path:
data/evidence.jsonl records decoded properties, subnode references, body payloads,
and attachment payloads with stable source references, hashes, bounded raw bytes, and
explicit failure statuses. The fixture workflow checks uniqueness, required evidence
kinds, and the raw-retention bound; semantic parity for every property and payload
family remains owned by the downstream work units.

## RP-M0-01 evidence contract

The matrix uses the shared status, provenance, fixture, and evidence types in `tests/readpst_diff/manifest.rs`. A row may be promoted only when its evidence level supports the claim; the approved Tika Unicode fixture remains E2/Partial baseline evidence and does not change any matrix row. Comparison reports retain the pinned readpst revision, input family, crypt method, output profile, worker count, observed outcomes, artifact digests, inventory counts, and repeat-run determinism result. Malformed, ambiguous, unsupported, unavailable, corrupt, and failed outcomes remain explicit rather than being collapsed into a successful extraction.

The RP-M0-03 source manifest pins every reviewed direct path and work-unit anchor to the same upstream revision. Source drift is a release-blocking evidence failure until the ledger and comparison oracle are explicitly reviewed; no matrix row is promoted from a moving upstream baseline.

The RP-M0-02 runner now provides the executable comparison contract: isolated bounded tool runs, semantic normalization, explicit parity/extension/unsupported/failure outcomes, negative path/resource checks, and repeated-report determinism. The approved Tika fixture remains E2/Partial evidence, and no capability row changes status from this control-plane work.

## RP-M1-01 input-capability delivery

The parser boundary now emits a typed capability projection before traversal. It classifies Unicode/ANSI/OST/unknown families and crypt method, records root/index/attribute readiness, applies the ISO-8859-1 fallback policy, serializes budgets, and fails closed for unsupported, malformed, partial, or over-budget input. The admitted ANSI Stage-A fixture proves a valid v14 header, root offsets, empty BBT/NBT leaf traversal, weak CRC/page-trailer validation, repeat determinism, and libpff acceptance. The separate Stage-B fixture adds one controlled folder/message/recipient/body/EML path; neither fixture claims broad ANSI compatibility.
## RP-M1-02 typed folder/item envelope delivery

The canonical extraction path now publishes `data/items.jsonl` with deterministic folder and item envelopes. Source node identity, folder ownership, canonical paths, associated visibility, item-kind confidence, raw evidence references, and unresolved/ambiguous/duplicate/path-collision statuses are explicit. This improves the ITEM-01/ITEM-02 evidence boundary; full mixed-folder, deleted-filter, and typed schedule/contact/calendar/journal/report corpus gates remain open even though bounded contact, appointment, and non-mail Partial projections now exist.

## RP-M1-03 classification and routing delivery

The canonical extraction path now classifies available `PR_MESSAGE_CLASS` evidence and records `routing_status` for ordinary notes, schedule emails, appointments, contacts, journals, reports, tasks, sticky notes, unknown classes, associated content, deleted content, and type-filter decisions. The default policy is fail-closed for missing/unknown visibility or class and excludes associated/deleted content without deleting its envelope. This is E1/E2 synthetic-policy evidence plus the existing public Unicode integration path; it does not promote a row to Implemented or claim CLI/output-adapter parity.

## RP-M2-01 delivery

The canonical message projection now exposes sent-representing/received-by native
address fields, four relevant date summaries, flags, importance, priority,
sensitivity, and receipt/report/delete controls. Unit evidence covers positive
values and absent controls; the Tika production verifier checks the additive field
shape and native-address preservation. This promotes the metadata evidence boundary
only; broad producer coverage, header normalization, and adapter parity remain open.

## RP-M2-02 delivery

The production path now emits `data/headers.jsonl` with one deterministic projection
per extracted message. It preserves exact stored header text, normalized RFC fields,
raw-property evidence links and bounded raw bytes, explicit Unicode/String8/default
charset policy, and non-authoritative statuses for malformed, ambiguous, absent, or
lossy header inputs. Unit evidence covers folded fields, body-fragment rejection,
String8 loss reporting, decode failure, raw retention, and stable identifiers; the
Tika fixture gates verify message-key coverage and authority safety. This advances
the header evidence boundary but does not claim final MIME adapter or code-page
conversion parity. The MAPI String8 decoder now selects supported message code-page/Internet CPID
metadata per property context, preserves raw code-page evidence and resolution provenance, rejects
conflicts/unsupported declarations, and honours `-C` as authoritative; broader producer code-page
parity remains Partial.

## RP-M2-03 delivery

Attachment records now publish deterministic method/source and rendering-position
fields through the production canonical path. Existing attachment and embedded
fixtures verify by-value and method-5 payload hashes, safe archive paths, CID/method
metadata, size mismatch visibility, and stable ordering. Reference/ambiguous/empty
payload outcomes remain explicit; this does not yet close all adapter-specific
extension filtering or OLE round-trip rows.

## RP-M2-04 delivery

The canonical extraction path now publishes deterministic `data/mime_parts.jsonl`
records over independent body candidates and attachment evidence. Plain/HTML/RTF
media types, selection order, transfer encoding, raw/decoded hashes, inline/CID
metadata, embedded ownership, bounded RTF validation, and derived `fromhtml1` HTML
are covered by unit tests and the repeat-run body/MIME workflow. Unresolved locators,
invalid RTF, encrypted bodies, reports, and schedules remain explicit
non-authoritative statuses; typed special-item and adapter parity rows are not yet
promoted.

## RP-M3-01 delivery

The production path now publishes deterministic `data/embedded_graph.jsonl` edges
for method-5 embedded-message references. The approved Tika fixture verifies exact
parent/attachment/child identity, source method, child payload hash, child MIME
ownership, evidence linkage, and repeat-run byte equality. Unit coverage exercises
cycle detection and depth-budget propagation; unavailable, non-email, duplicate-owner,
and budget outcomes are explicit non-authoritative records. ATT-05 and ATT-11 remain
Partial because broader input methods, special-item semantics, and adapter parity are
still downstream work.

## RP-M3-02 delivery

The production path now publishes deterministic `data/special_items.jsonl` records
for report, schedule/meeting, encrypted, and synthetic RTF branches. Raw readable
special-body hashes survive missing or malformed semantic properties; encrypted
records never expose decoded hashes; validated RTF emits a synthetic non-authoritative
`application/rtf` MIME part without replacing raw evidence. Synthetic unit and
repeat-run evidence uses the repository `sample.pst` RTF fixture and the existing
body/MIME fixtures, while broad report-type, recurrence, and calendar-property
coverage remains Partial.

## RP-M3-03 delivery

Typed CLI policy translation is integrated into the canonical path. `canonical`,
vCard, contact-list, the source-backed Partial `iCalendar` profile, and the Partial
`vjournal` profile are implemented; mbox, recursive mbox, MH, EML, separate, and KMail
are Partial adapter profiles with explicit filter/path/index decisions. Thunderbird and
MSG names are recognized and return explicit unsupported results until their dedicated
RP-M5 adapters land. Visibility/type filters
are applied to item routing statuses while source provenance remains present, and
policy JSON plus repeat-run item output are deterministic. Adapter parity remains open.

## RP-M4-01 delivery

Contact-class records now have a canonical `ContactRecord` projection and deterministic
vCard/contact-list serializers. Source-backed fields retain evidence and missing or
unvalidated contact properties remain explicit partial/unavailable statuses. The
java-libpst contact/distribution-list corpus is currently negative/partial because its
contact classes are not yet authoritative in PSTD; the positive serializer gate is a
provenance-labelled synthetic unit fixture. Full contact-field parity remains Partial.

## RP-M4-02 delivery

Appointment-class records now have a canonical `CalendarRecord` projection and a
deterministic `icalendar` profile. Source identity, class, subject, and available
organizer values retain evidence. Appointment dates, timezone, recurrence, exceptions,
alarms, and categories remain explicit unavailable statuses because the current
canonical property decoder does not yet authorize those MAPI groups. The profile uses
only a marked synthetic `DTSTAMP` for standards-shaped deterministic output; it never
guesses a source date. Schedule-email MIME remains RP-M3-02.

## RP-M4-03 delivery

`NonMailRecord` now preserves journal, task, sticky-note, unknown, and missing-class
items exactly once through canonical JSONL evidence, manifest entries, and raw body
references. Journal records expose a deterministic Partial `vjournal` profile. Task
and sticky-note records retain `skipped_unsupported_by_readpst` while recording
PSTD's stronger typed-preservation status; unknown and missing classes use distinct
PSTD-unsupported statuses. None are promoted to ordinary email.

## CLI, policy, and operational behaviour

| ID | Capability | Status | Closure evidence |
|---|---|---|---|
| CLI-01 | Help and version | Implemented | Stable help/version output exposes the canonical path, named profiles, supported policy flags, and the inspect/batch command surface; unknown options fail closed. |
| CLI-02 | Output directory | Implemented | All adapters share a safe output-root contract. |
| CLI-03 | Debug file, level, quiet mode | Partial | Bounded structured logs with severity filtering and deterministic paths. |
| CLI-04 | Parallel jobs | Implemented | Bounded batch scheduling preserves sorted item results and normalized progress semantics at `jobs=1` and a higher worker count; scheduling evidence is independent of PST content. |
| CLI-05 | Overwrite/unique output policy | Partial | Explicit policy in run config and adapter tests. |
| CLI-06 | Fallback charset (`-C`) | Partial | Validated `iso-8859-1`, `windows-1252`/`cp1252`, `utf-8`, Shift-JIS/`cp932`, GBK/`cp936`, EUC-KR/`cp949`, and Big5/`cp950` fallbacks reach message, folder, table-row attachment, and embedded-message property contexts; supported per-context `PR_MESSAGE_CODEPAGE`/`PR_INTERNET_CPID` values now select String8 decoding, with raw evidence/provenance, explicit malformed-conversion counts, and fail-closed conflicts. Broader item-level and producer corpus coverage remains open. |
| CLI-07 | Prefer UTF-8 (`-8`) | Partial | Explicit body/output encoding policy with raw-byte retention. |
| CLI-08 | Include deleted items (`-D`) | Gap | Deleted traversal, filter, counts, and scoped records. |
| CLI-09 | Item-type filter (`-t`) | Partial | Applied to every named mail/typed output projection through routed source identities; broad mixed-folder differential evidence remains. |
| CLI-10 | Attachment extension allow-list (`-a`) | Implemented | Normalized case-insensitive filtering applies across mailbox, separate, KMail, and MSG/EML projections; filtered decisions are explicit and canonical attachment records/payloads remain complete. |
| CLI-11 | Suppress synthetic RTF attachment (`-b`) | Partial | Adapter policy that does not erase canonical RTF evidence. |
| CLI-12 | Contact modes (`-cv`, `-cl`) | Partial | Source-backed vCard and simple contact-list profiles with explicit partial/empty status. |
| CLI-13 | Appointment iCalendar profile | Partial | Deterministic `icalendar` profile with explicit unavailable date/recurrence status. |
| CLI-14 | Journal vJournal profile | Partial | Deterministic `vjournal` profile with explicit source-field and synthetic-timestamp status. |

## Input and parser families

| ID | Capability | Status | Closure evidence |
|---|---|---|---|
| IN-01 | Unicode PST traversal | Partial | Multiple producer fixtures with folder/message/body/attachment completeness. |
| IN-02 | ANSI version 14/15 traversal | Partial | Admitted Linux Rust Stage-A v14 structural fixture plus Stage-B one-folder/one-message fixture and Stage-C direct/indirect method-1 plus method-2/method-3/method-4 attachment fixtures with exact populated BBT/NBT entries, flat ANSI String8 property contexts, contents/recipient/attachment heaps, independent byte validation, repeat-run equality, PSTD canonical records, exact plain-text EML, exact inline/external attachment evidence, and explicit method metadata; external ID2/path references, embedded/OLE producers, HTML/RTF, typed items, producer breadth, and malformed semantic derivatives remain open. |
| IN-03 | OST 2013 input | Partial | Controlled 4 KiB OST 2013 header/root/BBT/NBT traversal, explicit truncation/malformed negatives, and repeat-run evidence; full item/output corpus remains. |
| IN-04 | No/compressible/strong encryption | Partial | Method 0/1/2 production payload statuses, pinned method-1/method-2 vectors, method-2 capability readiness, unknown-method fail-closed evidence, and bounded decode limits; broad encrypted item/output differential corpus remains. |
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
| ITEM-03 | Mixed item types in one folder | Partial | Production routing statuses plus a synthetic mixed-folder class matrix; an admissible mixed PST and typed output adapters remain required. |
| ITEM-04 | Deleted and associated contents | Gap | Visible/deleted/associated filtering with explicit counts. |
| ITEM-05 | Ordinary note/email | Partial | Broader message metadata/body/attachment corpus. |
| ITEM-06 | Schedule/meeting email | Gap | Email plus validated `text/calendar` part. |
| ITEM-07 | Appointment/event | Partial | Typed appointment record and deterministic iCalendar profile; recurrence/date property corpus remains required. |
| ITEM-08 | Contact | Partial | Source-backed typed contact plus deterministic vCard/list output; full MAPI contact fields remain required. |
| ITEM-09 | Journal | Partial | Canonical `NonMailRecord` plus deterministic vJournal output; full MAPI field coverage remains required. |
| ITEM-10 | Sticky note/task/other classification | Partial | Typed non-mail preservation, distinct readpst/PSTD statuses, and deterministic evidence are integrated; dedicated renderers remain unsupported. |
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
| MSG-09 | RFC 2047 header encoding | Implemented | Generated non-ASCII subjects/display names use deterministic UTF-8 encoded words; injection inputs fail closed and the compatibility EML projection is covered. |
| MSG-10 | RFC 2231 filename encoding | Implemented | Generated MIME name/filename parameters use deterministic UTF-8 percent encoding with ASCII fallbacks and continuations; normalized filenames are covered in both EML and MSG compatibility projections. |
| BODY-01 | Plain text | Partial | Charset and body-only producer coverage, including validated String8 code-page selection and four common East Asian code pages; broader item-level and producer coverage remains open. |
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
| ATT-01 | Long/short filename selection | Implemented | Canonical property projection prefers `PR_ATTACH_LONG_FILENAME`, falls back to `PR_ATTACH_FILENAME`, preserves the original candidate, and applies deterministic safe-name handling. |
| ATT-02 | MIME tag/default type | Implemented | Generated EML and canonical MIME parts use the attachment MIME tag, default safely to `application/octet-stream`, reject injected media types, and render deterministically. |
| ATT-03 | By-value method 1 | Partial | Direct and arbitrary-format payloads, 4/8-byte data trees, zero length, size mismatch, and unnamed attachment fallback are covered; #579 proves one deterministic ANSI v14 property-context direct-binary layout and #580 adds an ANSI HNID/SLBLOCK-indirect layout, each with exact canonical, inline MIME/base64, external-file/manifest, hash, and fail-closed truncated-input evidence. Broad producer/layout corpus remains. |
| ATT-04 | By-reference methods 2/3/4 | Partial | #581, #582, and #583 prove deterministic ANSI method-2, method-3, and method-4 property-context HNID/SLBLOCK mappings; #584 wires the same validated reader/BBT/data-tree resolver into attachment Table Context rows; #585 validates an external-ID2-style low-nibble-0xf mapping with exact payload/hash, MIME, external-file/manifest, repeatability, and fail-closed truncation evidence. Broader external path layouts and reference-producer corpus remain. |
| ATT-05 | Embedded message method 5 | Partial | Child attachment and nested-child recovery is bounded; ambiguous, non-email, and broad producer cases remain. |
| ATT-06 | OLE method 6 | Partial | Direct object bytes plus bounded property-context indirect wide/compact SLBLOCK resolution preserve exact arbitrary OLE/CFB bytes and explicit provenance; broader OLE producers, ID2 layouts, and output-profile evidence remain. |
| ATT-07 | Content-ID/inline correlation | Partial | Unique, missing, duplicate, and unmatched CID cases. |
| ATT-08 | Rendering position/MIME sequence | Partial | Canonical fields and MIME-sequence-first ordering are implemented; producer/differential evidence remains. |
| ATT-09 | Attachment extension filter | Implemented | Case-insensitive filter/status decisions cover every mailbox profile plus MSG/EML projections without mutating canonical attachment records or payloads. |
| ATT-10 | Synthetic RTF/encrypted body attachments | Gap | Synthetic-source markers and policy tests. |
| ATT-11 | Nested ownership and recursion limits | Partial | Deterministic parent/child graph and bounded recursion. |

## Output formats

| ID | Capability | Status | Closure evidence |
|---|---|---|---|
| OUT-01 | Canonical structured TAR/JSONL | Implemented | Existing output contract and deterministic fixtures. |
| OUT-02 | Default per-folder mbox | Partial | Deterministic canonical-record mbox stream, mboxrd unit evidence, and sample-PST repeat run; full semantic differential remains. |
| OUT-03 | Recursive folder mbox | Partial | Sanitized folder tree, stable collision suffixes, explicit decisions, and sample-PST repeat run; reduced typed streams remain. |
| OUT-04 | MH/rfc822 separate files | Partial | Folder-local numbered files with no separator and missing-body negative evidence; full corpus remains. |
| OUT-05 | Extended `.eml` output | Partial | Header normalization/reconstruction, multipart alternative, attachment projection, and sample-PST repeat run; full MIME differential remains. |
| OUT-06 | Separate binary attachments | Partial | Integrity-validated payload files including zero-length and recovered embedded-message bytes, normalized extension filters, collision-safe names, explicit negative decisions, and deterministic adapter manifest; broad corpus remains. |
| OUT-07 | KMail layout | Partial | Safe `.<folder>.directory/<folder>.mbox` projection and explicit index policy; KMail import/read test remains. |
| OUT-08 | Thunderbird `.type`/`.size` sidecars | Partial | Recursive mbox plus canonical-identity `.type`/`.size` sidecars, typed vCard/iCalendar/vJournal/non-mail files, semantic parsing, negative type evidence, and repeat-run equality; exact import compatibility remains. |
| OUT-09 | vCard/list | Partial | Source-backed contact output and explicit partial/empty profile evidence; full field round trips remain required. |
| OUT-10 | vCalendar/vJournal | Partial | Deterministic appointment iCalendar and Partial vJournal profiles with explicit missing-field/status evidence. |
| OUT-11 | `.msg` writer | Partial | Rust-native CFB/OLE writer, deterministic `.msg` plus `.eml` companion, supported MAPI property map, recipient/attachment storages, explicit unsupported/method-5 decisions, and independent `olefile` round-trip evidence; named properties and embedded breadth remain. |

## Matrix rule

The overall project cannot claim readpst parity while any applicable row is Gap or Partial. Rows that readpst itself skips still require an explicit PSTD classification and status so the skipped content is visible and auditable.

## RP-M7-01 promotion report — reviewed 21 August 2026

Review basis: PSTD `main` at `57fbcaf1a83e2ddc79fff300be812a23cc66bb53`, the pinned
libpst/readpst revision `cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89`, the source ledger,
and the merged evidence packets for issues #497–#521. The matrix was reviewed row by
row against the E2/E3/E4 definition of done; no row was promoted from a field accessor,
helper, fixture-only probe, successful process exit, or empty output.

| Promotion class | Rows | Decision |
|---|---:|---|
| Implemented | 2 | `CLI-02`, `OUT-01` retain current Implemented status with production-path and deterministic output evidence. |
| Partial | 54 | Retain Partial: the canonical boundary and selected fixtures are evidenced, but broad readpst input, item, semantic, or adapter coverage is not proven. |
| Gap | 19 | Retain Gap: no equivalent user-visible behaviour is currently proven. These rows block a full-parity claim. |

Evidence is grouped by the merged work-unit packets: control/evidence #497–#500;
typed extraction #501–#504; Unicode/message fidelity #505–#508; relationships and
special items #509–#511; non-mail outputs #512–#514; output adapters #515–#518;
input breadth and hardening #519–#521. Each packet records source anchors, production
modules, fixture provenance, positive and negative cases, deterministic-repeat results,
CI/workflow evidence, and remaining limitations. The RP-M6 workflows additionally
record controlled ANSI v14/v15 and OST 2013 derivatives, method-1/method-2 crypto
vectors, symlink/path/TAR/diagnostic negatives, bounded-worker equality, and the
reviewed run IDs in #519–#521.

The unresolved Gap set is intentionally explicit for the reviewed RP-M7-01 baseline:

`CLI-06`, `CLI-08`, `CLI-09`, `CLI-10`, `ITEM-04`, `ITEM-06`, `ITEM-11`, `MSG-04`,
`MSG-05`, `MSG-09`, `MSG-10`, `BODY-05`, `BODY-06`, `BODY-08`, `ATT-04`, `ATT-06`,
`ATT-08`, `ATT-09`, and `ATT-10`.

These are not reclassified as “unsupported by readpst”; the source review confirms
that readpst exposes the corresponding command, item, body, attachment, or output
behaviour. The remaining Partial rows likewise remain open until representative
semantic differentials and full output/import evidence exist. RP-M7-02 must consume
this exact baseline; RP-M7-03 must use its unresolved set in the final decision.

The independent E4 report is [`rp-m7-02-e4-report.md`](rp-m7-02-e4-report.md). It
records the pinned-oracle run, output/profile coverage, repeat-run requirement, and
admissibility blockers without promoting a row.

## RP-M7-03 final decision

The final release decision is [`rp-m7-03-parity-decision.md`](rp-m7-03-parity-decision.md):
**not parity-complete** for its reviewed baseline of 2 Implemented, 54 Partial, and 19 Gap.

## Post-RP-M7 output parity expansion — 22 August 2026

The first post-decision implementation slice was merged to `main` via PR #554. It
keeps canonical `messages.jsonl`, typed
records, payloads, and item-routing evidence complete while applying the requested
projection policy only to named outputs:

- `-t[eajc]` routed source identities now select mailbox, MSG/EML, vCard,
  iCalendar, vJournal, and Thunderbird typed projections; filtered items remain in
  canonical JSONL with their routing statuses and folder counts.
- `-a` extension allow-lists now apply to every generated mail/MSG MIME projection,
  not only separate attachment files. Filter decisions remain explicit and
  case-insensitive; canonical attachment records and payloads are not deleted.
- Generated subjects/display names use deterministic RFC 2047 UTF-8 encoded words.
  Generated MIME `name`/`filename` parameters use RFC 2231 UTF-8 percent encoding,
  ASCII fallbacks, and safe continuations for long values.

Focused tests cover routed identity selection, repeated output, extension filtering,
header encoding, filename encoding, and unchanged ASCII output. That slice moved five
rows from Gap to Partial; broad mixed-folder/input corpus and independent readpst/import
differentials remain required for those broader claims.

That initial slice changed the intermediate ledger to **2 Implemented, 59 Partial,
and 14 Gap**. The first closure wave brought the maintained ledger to **8
Implemented, 55 Partial, and 14 Gap**. The attachment metadata closure below brings
it to **10 Implemented, 53 Partial, and 14 Gap**. The attachment payload extraction
wave then moves three attachment capability rows from Gap to Partial, bringing the
maintained matrix to **10 Implemented, 54 Partial, and 11 Gap**. The RP-M7-03
document remains the historical release decision for its reviewed baseline; the
current row states above are the maintained ledger after these expansions.

## Easiest Partial closure wave — 22 August 2026

The following six rows were promoted without requiring a public PST fixture for
each row because their claimed behaviour is either CLI policy or deterministic
projection logic rather than a new PST traversal claim:

- `CLI-01`: help/version, supported profile discovery, and unknown-option failure;
- `CLI-04`: bounded batch scheduling and normalized progress ordering at multiple
  worker counts;
- `CLI-10` and `ATT-09`: normalized extension filtering across every mailbox and
  MSG/EML projection while canonical attachment evidence remains unchanged;
- `MSG-09`: RFC 2047 encoding, deterministic output, and header-injection rejection;
- `MSG-10`: RFC 2231 encoding, normalized filename handling, deterministic output,
  and compatibility EML coverage.

The closure evidence is synthetic and canonical-record based where the behaviour is
pure projection policy. This is sufficient for these rows because it does not claim
new parser/input coverage; input breadth rows remain Partial until their corpus gates
are satisfied.

## Attachment metadata closure wave — 22 August 2026

The next two rows were promoted from canonical attachment metadata and projection
evidence:

- `ATT-01`: property-context projection prefers the long filename and falls back to
  the short filename before deterministic safe-name handling;
- `ATT-02`: generated EML and canonical MIME parts preserve safe MIME tags, use
  `application/octet-stream` for missing or unsafe values, and reject header injection.

Focused tests cover positive selection, short-name fallback, missing and invalid MIME
tags, repeated output equality, and unchanged payload bytes. This remains a metadata
and output-policy closure only; payload methods, CID correlation, reference resolution,
and broad PST corpus coverage remain Partial or Gap.

## Planned implementation — `RP-10`

The matrix is the release ledger, so each capability row must map to an implementation plan, concrete PSTD boundary, and fixture family. The following mapping is the minimum issue index; sub-issues may split a plan without changing the row ID.

| Matrix area | Plan IDs | PSTD boundary to implement | Required acceptance family |
|---|---|---|---|
| `CLI-*` | `RP-01` | `ReadpstProfile` translation, bounded scheduler, adapters, diagnostics, overwrite policy | CLI golden tests; same fixture at `jobs=1`/`jobs=N`; reruns/collision cases |
| `IN-*` | `RP-02` | family/crypt/charset/input evidence and parser limits | ANSI, Unicode, OST 2013, encrypted, sparse/large, corrupt derivatives |
| `ITEM-*` | `RP-03`, `RP-08` | `ItemEnvelope`, folder graph, visibility, typed classes | mixed-folder, deleted/associated, duplicate-name, unknown-class corpus |
| `MSG-*` | `RP-04` | metadata/header/address/date/flag evidence | folded/invalid headers; native/SMTP recipients; flags and FILETIME cases |
| `BODY-*` | `RP-05`, `RP-07` | `BodySet`, MIME tree, charset/RTF/report/schedule/encrypted projections | text/HTML/RTF/report/schedule/embedded MIME semantic corpus |
| `ATT-*` | `RP-06`, `RP-07` | `AttachmentResolver`, payload graph, CID/order, child edges | all methods, references, OLE, CID, filters, size/cycle failures |
| `OUT-*` | `RP-09` | mbox/MH/EML/KMail/Thunderbird/contact/calendar/MSG adapters | independent readers, hashes, paths, sidecars, OLE round trip |

### Promotion procedure

For every row, the issue and pull request must update the status only after these checks succeed:

1. The source function/constant and pinned libpst revision are cited in `12-upstream-source-notes.md`.
2. The canonical record can represent success, absence, filtering, unsupported, ambiguity, corruption, and failure.
3. The applicable output profile is implemented as a projection, not a renamed or reparsed substitute.
4. A positive fixture reaches the stated evidence level and a malformed/ambiguous fixture fails closed.
5. Semantic differential output matches readpst for the common boundary; stronger PSTD behaviour is documented as an intentional improvement.
6. Repeated runs and bounded worker counts produce equal canonical ordering, IDs, hashes, statuses, and path decisions.
7. Tangential documents—topic page, README, roadmap, source ledger, current-state docs, and changelog—agree with the new status.

### Implementation issue shape

The matrix should be maintained by a small generated or checked table rather than hand-editing status prose in isolation. Each row’s issue metadata should contain:

```text
matrix_id, plan_id, status, source_anchor, pstd_modules,
fixture_ids, differential_command, negative_fixture_ids,
output_profiles, evidence_level, documentation_fanout
```

When a parser change affects three or more areas, update this matrix first, then recursively inspect every linked plan page for stale status, module names, or acceptance boundaries. The issue template and comparator rules are in [RP-13](13-issue-template-and-differential-harness.md); the release sequencing is in [RP-11](11-roadmap-and-acceptance.md).
