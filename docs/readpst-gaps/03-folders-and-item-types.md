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
