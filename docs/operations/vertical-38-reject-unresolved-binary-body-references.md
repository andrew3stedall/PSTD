# Vertical 38: reject unresolved binary body references

## Objective

Stop emitting a four-byte Property Context locator as a binary message body while preserving every independently decoded body representation.

This milestone follows the independent baseline in PR #465. It does not resolve external HNIDs, infer one body form from another, or change MIME assembly.

## Root cause

Variable-length binary properties in a Property Context can occupy a four-byte HNID cell. The body extractor accepted any `MapiValue::Binary`, including that unresolved locator, as final payload bytes.

The pinned Tika fixtures expose two examples:

- `tika-various-body-types.pst`: `PR_HTML` reference `7f800000`;
- `tika-testPST.pst`: parent and embedded-child `PR_HTML` references, including `7f830000` on the child.

Those values are references, not HTML documents.

## Implementation boundary

Binary HTML and compressed-RTF body candidates are accepted only when they are:

- represented as a decoded `MapiValue::Binary`;
- non-empty;
- not the four-byte Property Context HNID cell;
- no larger than the 64 MiB body bound.

A present property that fails that boundary produces an explicit zero-byte unavailable `BodyRecord`. A valid sibling body remains extracted. When unresolved HTML and valid plain text coexist, the report selects `text` deterministically and leaves `has_html_body=false`.

## Exact body-types fixture result

`tika-various-body-types.pst` remains four messages and five body records.

For `msg_58f39dbbca78135b`:

- the 37-byte UTF-8 plain body remains extracted;
- the HTML form is a zero-byte unavailable record;
- no HTML payload file is written;
- `has_text_body=true`;
- `has_html_body=false`;
- preferred body type is `text`;
- unresolved body types are `html`.

The other three messages retain their explicit property-absent text records.

## Tika attachment regression result

`tika-testPST.pst` remains:

- 8 messages and 10 body records;
- 9 recipients;
- 2 attachment records and 12,315 attachment payload bytes;
- 2 EML files and 17,488 EML bytes;
- the same authoritative folder ownership for all seven top-level messages;
- the same isolated embedded-child relationship.

The corrected body evidence is:

- 6 body payload files / 271 bytes instead of 8 / 279;
- two zero-byte unavailable HTML records, for `msg_c6163b9157944cc9` and `msg_0ff529af59d373d5`;
- messages JSONL: 23,865 bytes;
- bodies JSONL: 2,922 bytes;
- extraction TAR: 234,496 bytes;
- total extraction output: 279,543 bytes.

The removed eight bytes were two unresolved four-byte references. No valid attachment, plain-text body, ownership field, or EML byte changed.

## Fail-closed behavior

- raw or enum-wrapped four-byte HNID cells are never materialized as body files;
- empty and over-limit binary body values remain unavailable;
- unresolved forms are recorded independently instead of suppressing valid siblings;
- top-level and embedded-message extraction use the same body-form policy;
- body flags reflect materialized payloads only.

## Validation

The draft PR runs:

- all Rust tests;
- clippy with warnings denied;
- rustfmt;
- Python packaging and CLI checks;
- Docker build;
- the exact Tika body-types fixture;
- the exact Tika attachment/ownership fixture;
- readable EML;
- readable RTF and recovered HTML.

## Next boundary

Establish the first pinned public ANSI PST baseline without weakening the approved Unicode fixture contracts. External body-HNID resolution remains deferred until a fixture can prove the owning heap/subnode context and exact resulting bytes.
