# Embedded and special email items

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
