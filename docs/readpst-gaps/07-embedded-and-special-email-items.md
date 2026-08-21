# Embedded and special email items

## RP-M1-04 provenance boundary

The canonical archive now exposes subnode references and embedded/body/attachment
payload provenance through data/evidence.jsonl. Evidence records preserve stable
owner keys, source references, bounded raw bytes, and explicit unavailable/failed
statuses; they do not imply that an unsupported embedded item has been decoded.

## Embedded RFC 822 messages

The readpst writer detects attachment method `PST_ATTACH_EMBEDDED`, changes its MIME type to `message/rfc822`, parses the referenced child item, and writes the child through the ordinary email writer. It carries selected header context from the outer message and skips/logs children that are not email items.

PSTD currently has an exact method-5 child path for one fixture. The parity gap is the general algorithm:

- resolve child identity without confusing the attachment node and child message node;
- recurse through child bodies, recipients, headers, and attachments;
- preserve parent/child ownership and ordinal relationships;
- support child HTML/RTF/report/schedule forms where validated;
- prevent cycles, duplicate ownership, and unbounded nesting;
- distinguish non-email embedded objects from malformed email children;
- emit a standalone child artefact and a parent `message/rfc822` payload only when both are independently validated;
- keep the parent MIME tree correct when a child cannot be materialized.

The fact that upstream logs and skips some embedded appointments is not a reason for PSTD to silently drop them. PSTD should classify the child as `embedded_appointment` or `embedded_non_email` and record `skipped_unsupported_type` when no readpst-equivalent email output exists.

## Meeting requests and responses

`PST_TYPE_SCHEDULE` is still an email item, but readpst adds a `text/calendar` body part. The schedule part can carry a method such as `REQUEST`, `REPLY`, or another source-provided method, and uses the sender and appointment data.

PSTD needs a typed schedule model that can preserve:

- method and calendar UID;
- organizer, sender, attendees, and recipient roles;
- start/end and timezone evidence;
- location, all-day, sequence, and recurrence;
- response status and meeting-specific MAPI properties;
- raw schedule bytes when present.

The ordinary email record and the calendar component must remain linked rather than one replacing the other.

## Delivery and disposition reports

The readpst writer selects `multipart/report` for `PST_TYPE_REPORT`, uses a report type, and can include report text. The email model also contains report time and NDR status, reason, diagnostic, and supplementary fields.

PSTD needs a report record and an EML adapter that can emit:

```text
multipart/report; report-type=<validated type>
├─ text/plain report text
└─ optional message/delivery-status or message/disposition-notification evidence
```

If the required report components are not available, the output must be marked partial/unavailable and the raw report properties retained.

## Encrypted body properties

libpst does not turn `encrypted_body` or `encrypted_htmlbody` into cleartext. readpst moves those bytes into attachment-like output so the source evidence is not lost. PSTD should provide the same lossless behaviour:

- preserve encrypted bytes as a dedicated body artefact or synthetic attachment;
- retain the source property and encryption status;
- never label the bytes as `text/plain` or `text/html` without a validated decode;
- make the output choice explicit in the manifest;
- keep encrypted and cleartext body status separate.

## RTF-body synthetic attachment

When enabled, readpst decompresses `PR_RTF_COMPRESSED` and adds an `application/rtf` attachment named `rtf-body.rtf`. This is distinct from a source attachment and must carry a synthetic-source marker in PSTD. The `-b` equivalent suppresses it from an output adapter but should not erase the raw RTF body from canonical extraction.

## Special handling and safe failure

The special-item path must test:

- embedded child with valid ordinary mail;
- child with HTML and RTF;
- child with by-value attachment;
- child with nested embedded message;
- child with appointment/schedule class;
- child with malformed object wrapper;
- missing ID2/data reference;
- cycle or duplicate child reference;
- encrypted child body;
- report child.

Each case needs a deterministic item graph and scoped diagnostics. Parent extraction must not be reported complete if a required child was silently lost.

## Planned implementation — `RP-07`

### Readpst logic reviewed

`write_embedded_message` resolves `attach->i_id`, obtains the descriptor/ID2 context, calls `pst_parse_item` on `attach->id2_head`, skips null or non-email children with diagnostics, forces `message/rfc822`, and recursively invokes `write_normal_email`. `write_normal_email` adds schedule parts for `PST_TYPE_SCHEDULE`, report MIME for `PST_TYPE_REPORT`, synthetic RTF/encrypted-body attachments, and recursively handles other embedded attachments. The upstream path has no general cycle/depth guard and can lose non-email embedded objects by design. `pst_process` recognizes DSN/MDN report properties and `pst_convert_recurrence` supplies appointment recurrence data.

### Planned PSTD graph

Add an explicit child graph to the typed envelope:

```text
EmbeddedGraph {
  nodes: ItemKey -> EmbeddedNode { class, source, status },
  edges: (parent_item, attachment_key, child_item, relation, ordinal),
  diagnostics: cycle | depth_limit | ambiguous_reference | non_email |
               missing_child | duplicate_owner | parse_failure,
}
```

`AttachmentRecord.embedded_message_key` and the `MetadataExtractionOutput` child payload maps are the compatibility starting points. Add `ScheduleRecord`, `ReportRecord`, and `OpaqueBodyRecord` rather than hiding them inside `MessageRecord`. `src/output/mime.rs` and typed calendar/report adapters consume graph edges; canonical JSONL/TAR stores every node and edge.

### Implementation flow

1. During `RP-06` resolution, register a child reference without recursing. Key it by parent item, attachment ordinal, source node, and resolved child identity.
2. Run a bounded graph expansion with a visited set, maximum depth, maximum nodes, and maximum payload budget. Detect cycles before parsing a child.
3. Parse and classify each child through `RP-03`. An email child can produce a `message/rfc822` MIME node; a contact/calendar/other child remains a typed child with explicit readpst-equivalent skip or stronger PSTD output.
4. Propagate parent header context only as a derived MIME field; never overwrite the child’s canonical metadata.
5. Build report and schedule records from typed properties and raw bytes. Validate report-type and calendar-method parameters before emitting MIME.
6. Move encrypted text/HTML and compressed RTF into opaque/synthetic artefacts with source tags, hashes, and profile-controlled output.
7. If a child fails, retain the edge, child source evidence, and reason. Mark the parent MIME/output projection partial or unavailable according to whether the child was required for the selected profile.
8. Emit standalone child records exactly once, then project the nested MIME relationship. Compare both parent and child semantic trees.

### Improvements over readpst

- Replace recursive function calls with a bounded graph plan and cycle detection.
- Preserve non-email embedded items instead of only warning and dropping them.
- Keep child canonical metadata independent from outer header context and avoid call-order mutation.
- Separate cleartext, encrypted, compressed RTF, report, schedule, and embedded-message statuses.
- Make partial parent output explicit when a required nested part cannot be materialized.
- Retain raw child bytes/properties and graph edges even when no legacy readpst output exists.

### Issue-ready acceptance

`RP-07A` covers embedded graph/reference expansion, `RP-07B` nested MIME, `RP-07C` schedule/meeting email, `RP-07D` report/disposition, and `RP-07E` encrypted/RTF synthetic artefacts. Acceptance fixtures must include valid nested mail, nested attachments, non-email child, cycle, duplicate reference, depth overflow, missing ID2, malformed child, schedule methods, report types, encrypted body, and compressed RTF. Assert parent/child ownership, graph termination, raw preservation, MIME part types, partial statuses, and deterministic output; fan out to [bodies](05-body-mime-and-rtf.md), [attachments](06-attachments.md), [non-mail outputs](08-contacts-calendar-journal.md), [storage](09-storage-and-interoperability.md), and the matrix.
# RP-M2-03 delivery

Attachment method-5 records now carry an explicit embedded-message source reference
and deterministic rendering position in addition to the existing child-message key,
archive path, declared/actual size, and payload hash. Reference failures remain
linked metadata and are not materialized as successful empty payloads.

## RP-M2-04 integration boundary

Method-5 attachments are projected as `embedded_message` MIME parts with explicit
parent ownership and child message keys. Missing child payloads remain
`mime_embedded_message_payload_unavailable`; recursive depth, cycle detection, and
typed child special-item semantics remain RP-M3 acceptance work.
