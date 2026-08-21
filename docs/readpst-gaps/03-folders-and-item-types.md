# Folders and item types

## Folder traversal

readpst starts from the message-store root, locates the top of the folder tree, recursively enters child folders, and processes every descriptor in each folder. It supports folders containing more than one item type and keeps output type streams separate so a calendar item is not placed into an email mbox.

PSTD currently emits folder inventory and message ownership records for validated Unicode fixtures. The following behaviours remain parity requirements:

- preserve the complete folder path and stable source node identity;
- preserve folder content, unread, associated-content, and child-folder counts;
- traverse mixed-type folders without choosing one folder-wide type;
- distinguish normal contents from associated/hidden contents;
- expose search-folder and hierarchy-table results where they affect visible item ownership;
- include or exclude deleted items by explicit policy;
- preserve the message-store root’s well-known folder references and store metadata;
- sanitize names only at the filesystem-output boundary, never in canonical records;
- report null descriptors, skipped children, and failed branches without losing the parent folder count.

## Item classes exposed by libpst

The item type constants in `libpst.h` are derived from `PR_MESSAGE_CLASS` or `PR_CONTAINER_CLASS`.

| libpst type | Typical meaning | readpst behaviour | PSTD status |
|---|---|---|---|
| `PST_TYPE_NOTE` | Ordinary mail or generic note. | Emits through the email/MIME path. | **Partial**: message path is fixture-validated, generic note handling is not. |
| `PST_TYPE_SCHEDULE` | Meeting request/response transported as email. | Emits an email plus a `text/calendar` schedule part. | **Gap** |
| `PST_TYPE_APPOINTMENT` | Calendar appointment/event. | Emits iCalendar. | **Gap** |
| `PST_TYPE_CONTACT` | Contact item. | Emits vCard or a simple list. | **Gap** |
| `PST_TYPE_JOURNAL` | Journal entry. | Emits vJournal. | **Gap** |
| `PST_TYPE_STICKYNOTE` | Sticky note. | Classified but falls through the normal readpst processing path. | **Explicitly unsupported by readpst**: PSTD must still classify and report it. |
| `PST_TYPE_TASK` | Task item. | Classified but not emitted by the current `readpst` process path. | **Explicitly unsupported by readpst**: preserve typed metadata and skip status. |
| `PST_TYPE_OTHER` | Other message class. | Classified; some versions route it through email-like handling, but unknown cases can be skipped. | **Partial**: preserve class and evidence; choose email-like output only with validated semantics. |
| `PST_TYPE_REPORT` | DSN/MDN/delivery report. | Emits through `multipart/report` when report fields are present. | **Gap** |
| message store/root | Store metadata, not a visible message. | Used to locate the folder tree; not emitted as mail. | **Partial**: root discovery exists; typed store output is not complete. |

## Deleted and associated content

The `-D` option includes deleted items. Associated contents have their own counts and NID class, and should not be silently treated as ordinary visible messages. PSTD must add:

```text
item_visibility = visible | deleted | associated | hidden | unknown
item_type       = note | schedule | appointment | contact | journal | sticky_note | task | report | other | store
```

The canonical result must retain excluded items as counts and, where parser evidence is sufficient, as metadata records with `extraction_status=skipped_deleted` or `skipped_associated`. Output adapters may omit them according to policy, but omission must be explainable from the manifest.

## Typed record requirement

Do not force non-mail items through `MessageRecord`. Add typed records or a tagged item envelope that can preserve:

- source folder and node identity;
- message/container class;
- common subject/body/comment/create/modify fields;
- type-specific fields;
- attachments and raw properties where present;
- extraction status and unsupported boundary.

This is necessary both for readpst parity and for avoiding the current failure mode where a parser can discover an object but the output layer has nowhere safe to put it.

## Planned implementation — `RP-03`

### Readpst logic reviewed

`readpst.c::process` receives a descriptor tree node, calls `pst_parse_item`, and routes by `item->type`: folders recurse; contacts go to `write_vcard`/list; ordinary notes, schedules, and reports go to `write_normal_email`; journals go to `write_journal`; appointments go to `write_appointment`; the message store is used only for root discovery; sticky notes, tasks, and unknown classes are skipped or warned. Deleted folders are excluded unless `-D`; child processing can be forked. `create_enter_dir` and `close_enter_dir` maintain separate reduced type streams and counts. `libpst.c::pst_process` maps MAPI message/container class values into the type constants and attaches the descriptor/item relationships.

### Planned typed envelope

Add an item boundary consumed by both metadata extraction and adapters:

```text
ItemEnvelope {
  source: { pst_id, descriptor_id, node_id, folder_id, ordinal },
  folder_path: canonical segments,
  visibility: visible | deleted | associated | hidden | unknown,
  kind: note | schedule | appointment | contact | journal | sticky_note |
        task | report | other | store,
  message_class: raw value + normalized class + confidence,
  common: subject/body/comment/create/modify evidence,
  typed: one of Message/Contact/Appointment/Journal/Report/Unknown,
  attachments: attachment keys,
  status: complete | partial | skipped_* | unavailable | corrupt | failed,
  raw_properties: property evidence references,
}
```

Extend `src/output/metadata.rs` with typed records and update `MetadataExtractionOutput` in `src/engine/metadata.rs` to carry an ordered `items` stream in addition to the existing message-compatible projections. `src/pst/folder_tree.rs`, `message_table.rs`, `message_ownership.rs`, and `message_metadata.rs` are the starting points; no output adapter should classify an item from a display name.

### Implementation flow

1. Discover the message-store root and folder candidates from validated NBT/Property Context evidence.
2. Build a folder graph keyed by source node identity, retaining canonical names, parent edges, content/unread/associated counts, search-folder kind, and rejected edges.
3. Enumerate contents and associated rows separately. Assign visibility before applying `RP-01` filters; an excluded item remains represented by a count or scoped record.
4. Parse common properties once, normalize the raw message/container class, and classify with a table-driven exact-match policy. Unknown classes remain `other` with raw evidence.
5. Emit one `ItemEnvelope` per source item. Never collapse two items because their subjects or display names match.
6. Route typed records to metadata and let output profiles select them. A mixed folder must produce independent email/calendar/contact/journal streams and preserve sibling failures.
7. Reconcile folder totals against item statuses and report null descriptors, missing child edges, duplicate ownership, and skipped classes.

### Improvements over readpst

- Keep deleted and associated items as explicit states instead of making inclusion a global side effect.
- Preserve mixed-folder ordering and source identity independently from reduced output streams.
- Detect duplicate/cyclic folder edges and report them rather than recursively entering indefinitely.
- Use a typed unknown/other envelope so unsupported classes are visible without forcing them through the email writer.
- Separate canonical folder names from sanitized output path names and preserve both.
- Expose item status and count reconciliation in JSONL/TAR; readpst’s `done/skipped/total` console counts are too coarse for a loss-auditable archive.

### Issue-ready acceptance

`RP-03A` should establish the envelope and status vocabulary; `RP-03B` should complete folder/ownership traversal; `RP-03C` should implement class/visibility routing; `RP-03D` should add mixed/deleted/associated fixtures. Acceptance requires:

- a mixed folder containing mail, schedule, appointment, contact, journal, task, sticky-note, report, and unknown classes;
- independent source and emitted counts for visible, deleted, and associated content;
- stable item identities under duplicate names and worker-count changes;
- explicit skipped statuses for classes readpst classifies but does not emit;
- no parent-folder suppression when one child or item is malformed;
- updates to [CLI policy](01-cli-and-output-parity.md), [metadata](04-message-metadata-and-headers.md), [non-mail outputs](08-contacts-calendar-journal.md), [storage](09-storage-and-interoperability.md), and the matrix.
