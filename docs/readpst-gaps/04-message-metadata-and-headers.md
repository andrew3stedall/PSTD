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

The readpst path applies RFC 2047 encoding to non-ASCII generated header values and RFC 2231 encoding to non-ASCII attachment filenames. PSTD needs a standards-tested header/parameter encoder for:

- display names and subjects containing non-ASCII characters;
- long filenames containing non-ASCII characters;
- quoted strings, backslashes, commas, semicolons, and control characters;
- line folding at safe boundaries;
- preserved raw headers that already contain encoded words.

The output must be deterministic and round-trip through a standards-compliant MIME parser.

## Date and status policy

PSTD must preserve every validated FILETIME as a typed timestamp and must not substitute the Unix epoch silently. When an EML adapter needs a date, it should use the same evidence hierarchy on every run and record the source field. Readpst’s fallback date is an output compatibility behaviour, not permission to claim that the PST contained a real date.
