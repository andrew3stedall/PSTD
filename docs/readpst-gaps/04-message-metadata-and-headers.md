# Message metadata and headers

## Metadata groups

The libpst email model exposes more than the current PSTD `MessageRecord`. The parity target is to preserve raw evidence and typed interpretations where they are validated.

| Group | readpst/libpst examples | PSTD status |
|---|---|---|
| Identity | message class, display name, record key, message size, source block/node identity. | **Partial** |
| Sender and representing parties | sender name/address/type, sent-representing name/address/type, received-by name/address/type. | **Partial**: one sender path is present; all representing/received identities are not. |
| Recipients | To/Cc/Bcc display strings plus row-level role, name, address, address type, and SMTP evidence. | **Partial**: row-aligned records are proven on fixtures; broad recipient-property coverage is not. |
| Dates | client submit/sent, delivery/arrival, creation, modification, report time. | **Partial**: validated Date and selected delivery evidence exist; all date fields are not emitted. |
| Threading | Message-ID, In-Reply-To, References, conversation topic/index, normalized subject. | **Partial** |
| State flags | read, unmodified, submitted, unsent, has attachment, from me, associated, resend, receipt pending. | **Gap**: only selected booleans are represented. |
| Delivery/report controls | importance, priority, sensitivity, read receipt, reply requested, delivery report, delete after submit. | **Gap** |
| Exchange forensic identities | search keys, received-by/representing fields, native Exchange addresses. | **Partial**: raw Exchange addresses are preserved where proved; the full field set is not. |

## RP-M2-01 delivery

The canonical MessageRecord now projects sent-representing and received-by native
addresses, client-submit/delivery/creation/modification FILETIME summaries,
importance, message flags, priority, sensitivity, and the read-receipt, reply,
delivery-report, and delete-after-submit controls when the source properties are
available. Missing controls remain null and the metadata status identifies whether
report controls were observed. Raw property bytes and decode status remain in the
canonical evidence stream; no native address is rewritten as SMTP.

## RP-M2-02 delivery

The canonical extraction path now emits `data/headers.jsonl` for every extracted
message, including embedded messages and unavailable property contexts. Each record
keeps the exact decoded stored header value, a deterministic LF-normalized header
projection, the selected Unicode/String8/default-charset policy, and a stable link
to the raw property evidence. The validator accepts folded RFC fields and fields
without a space after the colon, keeps valid stored headers authoritative, and marks
body fragments, malformed names, bare line endings, decode failures, and lossy raw
encodings non-authoritative without dropping their evidence. `-C`/`-8` remain an
explicit adapter-policy boundary; the current String8 decoder records its UTF-8
lossy status and ISO-8859-1 fallback policy rather than silently claiming code-page
equivalence.

## Transport headers

readpst prefers the stored `PR_TRANSPORT_MESSAGE_HEADERS` when it looks like a valid RFC 822 header block. It removes duplicated or container-only fields such as MIME-Version, Content-Type, Content-Transfer-Encoding, Content-class, X-MimeOLE, and some Outlook wrapper fields, then reconstructs missing `From`, `Subject`, `To`, `Cc`, `Date`, and `Message-Id` fields from MAPI values.

It also:

- tolerates folded headers and headers without a space after the colon;
- rejects body fragments that merely look like headers;
- preserves Bcc through an `X-libpst-forensic-bcc` header;
- preserves non-SMTP sender evidence through `X-libpst-forensic-sender`;
- adds `Status: RO` for read messages;
- derives a fallback sender from a valid `From:` header when the MAPI sender is not an SMTP address;
- carries embedded-message header context into the nested `message/rfc822` part.

PSTD currently stores transport headers and can assemble selected EML headers, but does not yet provide this complete normalization and forensic policy. The implementation should keep both:

```text
stored_transport_headers: exact validated source text
normalized_headers:       deterministic reconstructed output
forensic_fields:          structured values that do not belong in ordinary mail headers
```

No untrusted stored header may inject a new header line. Header folding, duplicate fields, invalid names, and CR/LF safety need dedicated tests.

## Address semantics

An address type is not an SMTP address. PSTD’s existing rule of preserving Exchange/X.500 evidence unless a unique authoritative SMTP mapping exists is correct and must be retained. Parity requires the same rule for sender, sent-representing, received-by, and every recipient row.

Required fields include the native value and type, a separately optional SMTP value, and a resolution status such as:

```text
smtp_available
raw_address_preserved
raw_address_without_type
address_unavailable
ambiguous_mapping
```

Do not copy readpst’s display-string flattening into the canonical record; provide it only as an output-adapter projection.

## Header encoding

The readpst path applies RFC 2047 encoding to non-ASCII generated header values and RFC 2231 encoding to non-ASCII attachment filenames. PSTD now has a focused standards-aware projection encoder for the generated mailbox/MSG MIME path, while the broader stored-header and reader corpus remains open. The complete parity target still requires:

- display names and subjects containing non-ASCII characters;
- long filenames containing non-ASCII characters;
- quoted strings, backslashes, commas, semicolons, and control characters;
- line folding at safe boundaries;
- preserved raw headers that already contain encoded words.

The output must be deterministic and round-trip through a standards-compliant MIME parser.

## Post-RP-M7 output encoding delivery

`src/output/headers.rs` provides deterministic UTF-8 RFC 2047 encoded words for generated
subjects and display names. The same output layer emits RFC 2231 `name`/`filename`
parameters with ASCII fallbacks and continuation segments for long values. The mailbox,
MSG compatibility EML, embedded-message EML, and standalone `pstd-eml` projections use
these helpers; ASCII values retain their existing wire form. Canonical UTF-8 records and
raw stored headers are not rewritten. Generated RFC 2047/2231 projection is Implemented,
including deterministic output and header-injection rejection; stored-header
normalization, folding, and independent MIME-reader differentials remain Partial.

## Date and status policy

PSTD must preserve every validated FILETIME as a typed timestamp and must not substitute the Unix epoch silently. When an EML adapter needs a date, it should use the same evidence hierarchy on every run and record the source field. Readpst’s fallback date is an output compatibility behaviour, not permission to claim that the PST contained a real date.

## Planned implementation — `RP-04`

### Readpst logic reviewed

`readpst.c::write_normal_email` validates `item->email->header` through `valid_headers`/`header_is_reasonable`, extracts fields with `header_has_field`, `header_get_field`, and `header_get_subfield`, strips container-owned duplicates, and then reconstructs missing fields from MAPI values. It adds `Status: RO`, forensic sender/Bcc headers, RFC 2047 encodings, a fallback sender, and a deterministic mbox separator. `libpst.c::pst_process` populates sender, representing-party, received-by, recipient-row, date, flag, delivery-control, report, and threading fields. The `.msg` writer consumes the same model and maps selected properties into ANSI MAPI property streams.

RP-M5-04 maps the canonical metadata boundary into real OLE property records: Unicode
message class/subject/sender/body/header/message IDs, typed scalar flags, FILETIME sent
dates, and recipient role storages. Invalid or absent scalar values remain explicit MSG
status decisions; they are never replaced with guessed defaults.

### Planned PSTD model

Build a provenance-preserving metadata layer above the current `MessageRecord`, `RecipientRecord`, `MessageReferenceRecord`, and selected MAPI property records:

```text
HeaderEvidence {
  stored_raw: bytes,
  validated_lines: Vec<HeaderLine>,
  normalized: Vec<HeaderField>,
  stripped_duplicates: Vec<HeaderField>,
  rejected_lines: Vec<HeaderIssue>,
}
AddressEvidence { native_value, address_type, smtp_value?, resolution_status }
FieldEvidence<T> { raw, value?, source_tag, source_encoding, status }
MessageMetadata { identities, recipients, dates, flags, controls, threading, headers }
```

Keep raw transport text separate from normalized output. Add typed identity fields for sender, sent-representing, received-by, reply-to, To/Cc/Bcc rows, and distribution-list rows. Expand flags and delivery controls from `libpst.h` without treating a missing property as false. Preserve each FILETIME with its source property and conversion status.

### Implementation flow

1. Decode each String8/StringUnicode MAPI value with `RP-02`’s charset resolver and store raw bytes, decoded value, and conversion result.
2. Parse stored transport headers as a bounded line sequence. Reject CR/LF injection, invalid names, body fragments, and overlong lines; retain the rejected evidence.
3. Validate/fold headers using readpst’s observable acceptance rules, then remove only fields that the selected adapter owns. Preserve user headers not on the strip list.
4. Fill missing ordinary headers from the typed metadata using an explicit source precedence. Encode generated non-ASCII values with RFC 2047 and fold at safe boundaries.
5. Project address rows without converting non-SMTP native addresses into SMTP. Produce display-string compatibility fields only in adapters.
6. Derive read state and control headers from typed flags. Record whether `Status: RO`, forensic sender, or forensic Bcc was generated and why.
7. Resolve dates with a fixed hierarchy, use UTC/locale rules explicitly, and retain the original FILETIME even when the output adapter needs a fallback.
8. Emit both a normalized header model and a semantic comparator view; use the same model for EML, mbox, MSG, and report/schedule envelopes.

### Improvements over readpst

- Treat header input as hostile bytes and use typed line parsing rather than mutable C strings and `strstr`-style matching.
- Preserve duplicate/stripped fields in provenance so compatibility cleanup is reversible and auditable.
- Keep raw Exchange/X.500 identity and SMTP resolution status separately; do not flatten address semantics.
- Avoid mutating `pst_item` while writing, which in upstream can make later projections depend on call order.
- Use an explicit RFC 2047/2231 library/encoder with round-trip tests, safe folding, and deterministic choice of encoded-word form.
- Do not use the current time as canonical evidence. If an output format requires a fallback timestamp, mark it synthetic and keep it out of the source metadata.

### Issue-ready acceptance

`RP-04A` covers metadata projection, `RP-04B` address/recipient rows, `RP-04C` header validation/reconstruction, `RP-04D` flags/controls/threading, and `RP-04E` dates/charset. Each issue needs:

- folded, malformed, duplicate, injected, and body-looking header fixtures;
- Unicode/ANSI/native-address/SMTP recipient cases with row alignment;
- all relevant flag and control combinations, including absent versus false;
- date values at epoch boundaries, invalid FILETIME, and timezone conversions;
- EML and MSG semantic checks plus raw evidence assertions;
- documentation fan-out to [bodies](05-body-mime-and-rtf.md), [attachments](06-attachments.md), [storage](09-storage-and-interoperability.md), [the matrix](10-parity-matrix.md), and the source ledger.

## RP-M2-04 integration boundary

The canonical MIME projection consumes the validated `HeaderProjectionRecord` and
does not reparse PST header bytes. Generated MIME ownership is represented by
semantic part records, leaving malformed, lossy, or unavailable stored headers
non-authoritative for downstream adapters.

## RP-M3-02 special-item header boundary

Special report, schedule, encrypted, and synthetic RTF records link to the same
message/header ownership graph. They do not overwrite source headers or invent report
type, calendar method, recurrence, or decoded encrypted values when the relevant MAPI
properties are unavailable.
