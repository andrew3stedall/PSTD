# Unreleased

_Last reviewed: 21 August 2026._

## Added

### Compatibility analysis

- A pinned `libpst`/`readpst` parity gap register under `docs/readpst-gaps/`, covering CLI modes, PST/OST and encryption inputs, item classes, metadata, MIME and RTF bodies, attachment methods, contacts, calendar/journal outputs, storage formats, and acceptance fixtures.
- Issue-ready `RP-00`–`RP-13` implementation plans, a complete direct-source review ledger, and a semantic differential harness contract for translating every reviewed `readpst` capability into Rust-native PSTD work.
- A dependency-aware readpst parity agent orchestrator with eight milestone definitions, 28 stable work-unit slices, specialist-agent roles, merge gates, and recursive documentation fan-out.
- Deterministic RP-M0-01 parity contracts and tests for matrix/evidence statuses, fixture provenance, SHA-256/size/family/crypt admission, negative outcomes, pinned source drift, and E2 Unicode baseline evidence.
- RP-M0-03 source manifest and deterministic drift report covering all direct readpst/libpst ledger paths, work-unit anchors, regression profiles, and the no-copy GPL boundary.
- RP-M0-02 bounded semantic differential runner with isolated readpst/PSTD execution, normalized comparison findings, explicit negative outcomes, deterministic reports, and a pinned CLI-oracle workflow.

### RP-M3-03

- Added typed readpst CLI policy translation with fail-closed profile and option validation.
- Integrated visibility/type routing filters and deterministic policy provenance into canonical extraction.
- Added negative and repeat-run CLI evidence coverage; legacy output adapters remain explicitly unsupported until their RP-M4/RP-M5 work units land.

### RP-M4-01

- Added canonical contact records plus deterministic vCard and contact-list projections over source-backed fields.
- Preserved explicit partial/unavailable contact status and evidence when the admitted corpus cannot authorize contact classes.

### RP-M4-02

- Added canonical appointment `CalendarRecord` values and a deterministic `icalendar` profile.
- Preserved explicit missing date/timezone/recurrence/exception/alarm statuses and marked the synthetic iCalendar `DTSTAMP`.

### RP-M4-03

- Added canonical `NonMailRecord` evidence for journals, tasks, sticky notes, unknown classes, and missing message classes.
- Added deterministic Partial `vjournal` output and distinct readpst/PSTD statuses for skipped, preserved, and unsupported non-mail items.

### RP-M5-01

- Added production mailbox projections over canonical records for default mbox, recursive mbox, MH/rfc822, extended EML, and separate numbered message files.
- Added deterministic mboxrd escaping, MIME-header normalization/reconstruction, nested plain/HTML alternatives, binary attachment projection, path-safe folder collision handling, per-message negative decisions, output hashes, and an adapter manifest.

### RP-M5-02

- Added separate binary attachment publication over canonical payload evidence, normalized case-insensitive extension filters, collision-safe `<message-file>-<filename>` names, and explicit filtered, embedded, unavailable, and zero-length decisions.
- Hardened attachment materialization across generated EML: `pstd-eml` now accepts
  `--attachment-mode inline|external`, includes every integrity-validated recovered
  payload in inline MIME (including zero-length and method-5 `message/rfc822` bytes),
  and externalizes raw bytes with safe manifest-linked paths and explicit unavailable
  or integrity-failed statuses.
- Added a deterministic KMail `.<folder>.directory/<folder>.mbox` profile with explicit parent-index invalidation policy and no mutable index output.
- Added repeated-profile, path-safety, positive payload, and negative attachment/KMail evidence workflow coverage; Thunderbird sidecars and MSG remain downstream.

### RP-M5-04

- Added a deterministic synthetic Unicode PST with three messages and five attachments, generated from pinned MIT-licensed EMLtoPST tooling without private mailbox data.
- Added `data/cid_references.jsonl` evidence for unique, duplicate, unmatched, invalid, and orphan inline-CID relationships, with explicit attachment ownership and authoritative extracted-payload status.
- Added byte-for-byte fixture reproduction and readpst comparison coverage for attachment counts, payload materialization, direct HTML bodies, and inline EML Content-ID output.
- Added validated transport-header recipient fallback and direct extracted HTML support to the EML assembly adapter while retaining fail-closed admission rules.

### RP-M5-03

- Added a deterministic Thunderbird profile with recursive mbox output, canonical-identity `.type` and readpst-compatible `.size` sidecars, and explicit unavailable folder-type evidence instead of guessed numeric values.
- Added independent typed contact, appointment, journal, task, sticky-note, unknown, and missing-class projections for Thunderbird, with vCard/iCalendar/vJournal/JSONL outputs and a stable adapter manifest.
- Added semantic sidecar/file parsing, negative typed-source, path-safety, and repeat-run evidence workflow coverage; MSG and input breadth remain downstream.

### RP-M5-04

- Added a Rust-native CFB/OLE `msg` output profile with deterministic root MAPI properties, Unicode strings, FILETIME/scalar flags, recipient storages, by-value attachment storages, and three empty NameID streams.
- Added a separate `.eml` companion for readpst `-m` compatibility, explicit invalid/missing/method-5/method-6/unsupported decisions, and canonical raw evidence preservation.
- Added an independent `olefile` round-trip workflow covering semantic properties, recipient roles, attachment payload hashes, path safety, negative statuses, and repeated-byte determinism.

### RP-M6-01

- Added production ANSI v14/v15 and OST 2013 layout dispatch for headers, roots, BBT/NBT pages, inspect, and canonical metadata loading.
- Added controlled byte-level positive and negative fixtures with exact SHA-256 records, explicit strong-crypt/truncation/malformed statuses, and repeated inspect equality in `readpst-ansi-ost.yml`.
- The input rows are promoted from Gap to Partial structural coverage only; broader semantic family corpora, encryption, hardening, and release-gate promotion remain open.

### RP-M6-02

- Added a production table-driven crypto module for libpst methods 0, 1, and 2, including the pinned substitution permutation and block-ID salted strong transform.
- Added canonical payload-loader integration, machine-readable decode statuses, bounded pre-decode block limits, strong known-vector evidence, repeated decode equality, and explicit unknown-method failures.
- Promoted method-2 capability classification to `ready` when roots are safe; the controlled ANSI/OST workflow now records method-2 readiness and unknown-method unsupported evidence. The pinned NDB methods do not accept passwords, so password validation is not claimed.

### RP-M6-03

- Added a bounded batch worker pool with sorted result reassembly for independent continue-on-error runs, plus explicit symlink-safe and depth/file-count-bounded PST discovery.
- Added archive path confinement and close-then-rename TAR shard publication so incomplete shards remain temporary rather than appearing as final output.
- Enforced the diagnostic budget with an explicit `diagnostics_truncated` status and added malformed, symlink, path-safety, and one-worker/four-worker evidence.

### Product foundation

- Rust `pstd` CLI with `inspect`, `extract`, `batch`, and `version` commands.
- Python operator wrapper and Docker packaging.
- Structured TAR/JSONL output, stable identifiers, run summaries, progress logs, batch checkpoints, resume-by-skip behaviour, and operator handoff documentation.
- Folder, message, body, recipient, threading, attachment, selected-property, manifest, error, and summary record foundations.

### Parser and extraction

- RP-M1-04 canonical evidence stream with stable property, subnode, body-payload,
  and attachment-payload provenance, bounded raw retention, hashes, and explicit
  unavailable/failed statuses.
- RP-M2-01 additive message metadata projection for native representing/received-by
  identities, source dates, flags, importance, priority, sensitivity, and receipt/
  report controls with null-preserving missing-field semantics.
- RP-M2-02 canonical `data/headers.jsonl` projection with readpst-aligned stored-header
  authority checks, folded-field normalization, raw-property evidence links,
  explicit Unicode/String8/default charset policy, and non-authoritative malformed or
  lossy statuses.
- RP-M2-03 attachment source/method and deterministic rendering-position fields for
  by-value and embedded-message records, with safe-path, CID, size-status, and raw
  payload-hash evidence retained in canonical output.
- RP-M2-04 canonical MIME-part projection over body/attachment evidence, bounded
  direct/LZFU RTF validation, validated fromhtml recovery, and repeat-run MIME JSONL
  determinism with explicit unresolved/special-body statuses.
- RP-M3-01 bounded method-5 embedded-message graph projection with deterministic
  parent/child ownership, child evidence links, depth/node/byte limits, cycle
detection, and explicit unavailable/ambiguous/non-email statuses.
- RP-M3-02 typed report, schedule/meeting, encrypted-body, and synthetic RTF records
  with raw-evidence preservation, no encrypted decoded hashes, and deterministic
  special MIME projections.
- Typed input capability envelopes for Unicode/ANSI/OST/unknown family classification, crypt/root readiness, explicit unsupported/malformed/budget statuses, bounded reader limits, inspect visibility, and canonical extraction publication.
- Typed folder/item envelopes with source-ID ownership, visibility, item-kind confidence, parent/child links, explicit ambiguity/duplicate/path-collision statuses, and canonical `data/items.jsonl` output.
- Class-aware canonical routing statuses for ordinary, schedule, appointment, contact, journal, report, task, sticky-note, unknown, associated, deleted, filtered, and unavailable items, with immutable policy tests and no silent ordinary-message fallback.
- Safe PST header/root selection, bounded byte reads, checked arithmetic, BBT/NBT traversal, block and subnode access, depth/cycle guards, Heap-on-Node, BTH, Property Context, and Table Context parsing.
- Public PST progress workflow and deterministic bounded artifacts.
- Table Context descriptor evidence, bitmap mapping, row-payload candidate resolution, direct/ordinal addressing, validated row transport, fixed-width scalar decoding, and production diagnostics through PQ74.
- End-to-end recipient extraction from Table Context rows, including:
  - `PidTagRecipientType` role interpretation;
  - display-name, native email-address, and SMTP-address string resolution;
  - HNID and heap-resident value handling;
  - fail-closed row alignment and table attribution;
  - four structured `RecipientRecord` rows in production output.
- Readable EML assembly from validated message metadata, recipients, transport Date, Message-ID, and body payloads.
- Validated standalone RTF extraction for direct, MELA, and LZFu representations.
- HTML recovery from validated `\fromhtml1` RTF with bounded destination handling.
- Deterministic 956-byte `multipart/alternative` EML output with ordered `text/plain` and `text/html` parts.
- Three pinned upstream PST fixtures for attachment, multi-message, body-type, calendar, recurrence, contact, distribution-list, and Exchange-address validation.
- Filename-bearing attachment metadata extraction from validated recursive heap Property Contexts.
- Exact attachment data-NID to loaded data-BID resolution through validated Unicode SLBLOCK entries.
- Bounded Unicode XBLOCK decoding with ordered external child-BID resolution, exact `lcbTotal` assembly, duplicate/internal-child rejection, and DOCX signature validation.
- One validated 11,862-byte `attachment.docx` payload with deterministic archive path and SHA-256 `0c87a742c970907d3b08c73e7834768abadd00fe4f4995a7dd98a206d4c494c0`.
- A permanent Tika attachment fixture workflow that asserts exact metadata, payload bytes, checksum, ZIP CRCs, expected DOCX text, counts, and output bytes.
- Heap-backed Table Context row-matrix resolution through the owning Heap-on-Node allocation, reusing the existing bounded row and recipient projections.
- Direct root-SLBLOCK recipient-table attribution that excludes nested embedded-message tables from the outer message.
- Eight exact Tika recipient records across seven messages: six authoritative SMTP rows and two preserved raw/native rows, including a complete legacy Exchange distinguished name.
- One deterministic 17,035-byte Tika `multipart/mixed` EML with validated Date fallback, 22-byte plain body, raw/native addresses, and the byte-identical DOCX payload.
- Specification-aligned PtypObject handling that preserves the object HID, validates the exact eight-byte `Nid + ulSize` wrapper, and requires a normal-message NID.
- One separately keyed method-`5` child message linked through `embedded_message_key`, with one directly owned recipient, a 23-byte text body, and explicit unavailable HTML evidence for the unresolved four-byte property reference.
- A permanent Vertical 34 fixture contract covering exact child/parent ownership, stable attachment ordinals, record bytes, archive bytes, and unchanged outer EML.
- One deterministic 453-byte attachmentless `text/plain` EML for the linked method-`5` child, with exact SHA-256, headers, CRLF body, and exclusion of raw HTML bytes.
- Policy-gated plain-text-only EML admission from authoritative attachment metadata, retaining fail-closed behaviour for unrelated unvalidated plain-only messages.
- One exact 453-byte method-`5` `message/rfc822` payload, byte-identical to the standalone child EML, with stable path, key, ordinal, parent ownership and SHA-256.
- Shared plain-text EML construction plus focused rejection tests for missing, mismatched, duplicate, nested, ambiguous-body and unsafe-header cases.
- Exact physical contents-table membership decoding and fail-closed message-folder ownership reduction.
- Permanent Tika assertions for all eight folder records, seven top-level message owners and subjects, and the separate embedded-child boundary.
- Bounded binary-body admission that rejects four-byte Property Context HNID cells instead of materializing them as HTML or RTF payloads.
- Explicit unavailable body-form records plus deterministic valid-sibling selection for both top-level and embedded messages.

## Changed

- Shifted the active development model from milestone/PQ infrastructure work to evidence-led vertical extraction milestones.
- Prioritised extraction correctness and observable email fields over downstream Snowflake, UI, search, analytics, or graph implementation.
- Corrected B-tree page metadata and child-reference traversal.
- Decoded permitted `NDB_CRYPT_PERMUTE` blocks while preserving internal blocks as raw.
- Tightened payload admission so structurally invalid table declarations fail closed.
- Increased selected original-fixture properties from 0 to 16 and reduced unknown properties from 74 to 19.
- Recovered original-fixture text and RTF body payloads and eliminated the former fallback body row.
- Replaced legacy table assumptions with the real TC heap, row-index BTH, subnode-backed row storage, and four validated 52-byte rows.
- Prevented internal LTP row bookkeeping properties from being reported as user-readable fields.
- Preferred authoritative SMTP/native address properties over display-name fallback while retaining display names separately at the complete-record boundary.
- Replaced the original fixture EML's raw `text/rtf` alternative with validated recovered `text/html` while retaining plain text and all validated headers.
- Marked the Tika DOCX-bearing message as attachment-bearing from validated recursive Property Context evidence even though its direct message context omits `PidTagHasAttachments`.
- Resolved the Tika attachment HNID `0x0000833f` to loaded data BID `0x632` without treating the internal block as DOCX bytes.
- Corrected the earlier assumption that `PidTagAttachSize` had to equal the file payload: the fixture preserves 15,503 bytes as attachment metadata while the XBLOCK authoritatively emits 11,862 payload bytes.
- Suppressed unrelated attachment-table fallback rows once the validated filename-bearing Property Context attachment path is selected.
- Rebuilt the root README and current-state documentation so historical milestone/PQ files are no longer presented as the live roadmap.
- Prevented the attachment owner's recursively loaded subnode tree from emitting the same recipient projection twice when attachment presence is inferred from that tree.
- Preserved existing by-value attachment ordinals before appending method-`5` metadata, keeping the proven DOCX key and path stable.
- Replaced the invalid attachment-leaf-BID ownership assumption with the PtypObject's authoritative child NID, admitted only when that NID resolves exactly once within the outer message scope.
- Isolated the recovered child's loaded subnode subtree before reusing direct recipient projection, preventing child rows, bodies, and identifiers from leaking into the parent.
- Corrected PST table NID classification so `0x0e`/`0x0f` are physical contents tables, `0x10` is search contents, and hierarchy, attachment, and recipient tables cannot establish message ownership.
- Replaced synthetic-root ownership for all seven top-level Tika messages with the exact `/Début du fichier de données Outlook` folder from `node_802e` row keys.
- Reclassified two four-byte `PidTagHtml` Property Context locators as unresolved forms, removed their eight non-HTML payload bytes, and retained both valid plain-text bodies and byte-identical EML output.

## Current original-fixture result

```text
BBT/NBT entries: 50/63
Folders: 11
True/extracted messages: 1/1
Body payloads: 2
Attachments: 0
Selected/unknown properties: 16/19
Validated Table Context rows: 4 x 52 bytes
Structured recipient records: 4
EML files: 1
EML MIME alternatives: text/plain, text/html
EML bytes: 956
Standalone RTF bytes: 320
Standalone HTML bytes: 95
```

The original public fixture produces one readable email containing sender, To/Cc recipients, subject, Date, Message-ID, plain text, and recovered HTML.

## Current Tika attachment result

```text
Messages: 8
Body records: 10
Body payload files/bytes: 6/271
Recipients: 9
SMTP/raw-native recipients: 6/3
Attachment records: 2
Attachment payload files/bytes: 2/12315
DOCX SHA-256: 0c87a742c970907d3b08c73e7834768abadd00fe4f4995a7dd98a206d4c494c0
DOCX attachment ordinal/key: 0/att_0695091e19397627
Embedded attachment ordinal/key: 1/att_a9c94a13d70f1cb3
Embedded message key/NID: msg_0ff529af59d373d5/0x00200104
Embedded child sent FILETIME: filetime:132509026807730000
Embedded child text/unresolved-HTML payload bytes: 23/0
EML files/bytes: 2/17488
Messages JSONL bytes: 28727
Bodies JSONL bytes: 2922
Recipient JSONL bytes: 2708
Attachment JSONL bytes: 1240
Extraction TAR bytes: 525824
Total output bytes: 570872
```

All seven top-level messages belong to `/Début du fichier de données Outlook` through exact physical contents-table rows. The method-`5` attachment belongs to `msg_c6163b9157944cc9`, links to the separately emitted child, and publishes the exact child EML bytes at its existing archive path. The child owns its recipient and body records; both child and parent retain their valid plain-text bodies and explicit unavailable HTML forms. The parent retains only its direct recipient, DOCX, and unchanged EML.

## In progress

- Establish the first pinned public ANSI PST baseline without weakening the Unicode fixture contracts.

## Known limitations

- PSTD is not yet a generally compatible PST converter or PST-to-EML tool.
- The Tika fixture has exact parent and child EMLs plus one exact method-`5` payload, but broader producer/layout coverage remains unproven.
- One method-`5` child layout is validated; nested child attachments, recursion, and broad layout coverage remain deferred.
- ANSI, uncommon, corrupt, nested embedded-message, and broad MAPI-layout coverage remain incomplete.
- Non-ASCII RFC 2047 header encoding remains incomplete.
- Downstream Snowflake, UI, search, semantic search, graph, and LLM/RAG work remains parked.

## Removed or superseded

- The earlier assumption that completing M1-M25 made the extraction engine release-complete.
- The PQ-cycle roadmap as the default operating model after the validated parser foundation reached PQ74.
- Stale documentation that described recipient complete-record publication or first readable EML assembly as unfinished.
- Raw `text/rtf` as the preferred rich EML alternative for the current HTML-derived fixture body.
- The earlier evidence blocker that no approved attachment-bearing PST was available.
- The assumption that the attachment file payload must be padded or truncated to the 15,503-byte `PidTagAttachSize` value.
### Post-RP-M7 output parity expansion

- Applied routed `-t[eajc]` identities to mailbox, MSG/EML, vCard, iCalendar,
  vJournal, and Thunderbird typed projections while preserving complete canonical
  records and routing counts.
- Extended case-insensitive `-a` attachment filtering from separate files to generated
  mailbox/MSG MIME output, with explicit filtered decisions and unchanged canonical
  payload evidence.
- Added shared deterministic RFC 2047 UTF-8 encoded-word and RFC 2231 MIME parameter
  projection helpers, including ASCII fallbacks and long-value continuations, and used
  them in generated mailbox, embedded, MSG compatibility, and standalone EML output.
- Added focused regression tests for typed projection selection, extension filtering,
  ASCII stability, non-ASCII headers, and long filenames. The intermediate ledger was
  2 Implemented, 59 Partial, and 14 Gap; no full-parity claim is made.

### Easiest Partial closure wave — 22 August 2026

- Promoted `CLI-01` and `CLI-04` after stable help/version, fail-closed option,
  bounded-worker, and normalized progress evidence passed.
- Promoted `CLI-10`, `MSG-09`, `MSG-10`, and `ATT-09` after deterministic canonical
  projection tests covered normalized attachment filtering, RFC 2047/2231 output,
  header-injection rejection, compatibility EML, and all mailbox profiles.
- The maintained ledger is now 8 Implemented, 55 Partial, and 14 Gap; no full-parity
  claim is made.

### Attachment metadata closure wave — 22 August 2026

- Promoted `ATT-01` after direct canonical property-context evidence proved long-name
  preference and short-name fallback with deterministic safe output names.
- Promoted `ATT-02` after generated EML and canonical MIME projection proved MIME-tag
  preservation, octet-stream defaults, unsafe-value rejection, and repeat-run equality.
- The maintained ledger is now 10 Implemented, 53 Partial, and 14 Gap; no full-parity
  claim is made.

### Attachment payload extraction wave — 22 August 2026

- Replaced the DOCX-signature-only attachment loader with a bounded generic resolver
  for direct payload blocks and 0x0101/0x0201 data trees, including validated 4-byte
  and 8-byte child BID layouts, exact byte concatenation, cycle/repeat checks, and
  declared-size diagnostics.
- Added direct `PR_ATTACH_DATA_BIN`/`PR_ATTACH_DATA_OBJ` extraction, method-aware
  reference handling, hidden/rendering-position/MIME-sequence projection, and MIME
  sequence ordering. Attachment property contexts now retain unnamed, zero-length,
  and unresolved metadata rows instead of dropping them.
- Embedded-message recovery now walks child attachment subnodes recursively within
  the existing depth budget. Broad producer, OLE-reference, CID-correlation, and
  differential fixture coverage remain Partial. The maintained matrix is now 10
  Implemented, 54 Partial, and 11 Gap; no full-parity claim is made.

## RP-M7 release-gate review

- Added the RP-M7-01 conservative matrix promotion report at main commit `57fbcaf1a83e2ddc79fff300be812a23cc66bb53`: 2 Implemented, 54 Partial, and 19 Gap rows remain explicit.
- Preserved the distinction between readpst-exposed gaps and cases readpst itself skips; no row was promoted from a helper, field, fixture-only probe, or process exit.
- The unresolved Gap set is now the fixed input to the RP-M7-02 differential report and RP-M7-03 final decision.
- Added the RP-M7-02 E4 report and release workflow: the pinned oracle run is green for the approved Unicode fixture, while missing admissible corpus coverage remains explicitly not-proven rather than inferred.
- RP-M7-03 final decision: PSTD is not parity-complete at the reviewed baseline; 2 rows are Implemented, 54 Partial, and 19 Gap, with every remaining row and admissibility blocker named in the release decision.
- RP-M7-03 final decision: PSTD is not parity-complete at the reviewed baseline; 2 rows are Implemented, 54 Partial, and 19 Gap, with every remaining row and admissibility blocker named in the release decision.
