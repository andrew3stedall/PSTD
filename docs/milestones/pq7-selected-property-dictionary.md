# PQ7 Selected Property Dictionary Expansion

## Goal

Expand selected property-tag recognition for the true message candidate set after PQ6 showed a loaded property context with 74 unknown parsed properties.

## Scope

PQ7 is bounded to issues #269 through #273.

| Issue | Scope | Status |
|---:|---|---|
| #269 | PQ7 parent scope | Implemented in this PR |
| #270 | Unknown property tag diagnostics | Implemented |
| #271 | Safe selected MAPI property mapping | Implemented |
| #272 | Docs and public PST progress | Implemented; final artifact logged before merge |
| #273 | Validation and merge | Completed after final-head CI is green |

## Delivered behaviour

- ANSI/String8 variants of already selected Unicode MAPI string properties are now selected safely.
- String8 values decode as nul-terminated byte strings.
- Message metadata extraction checks Unicode and String8 aliases for subject, sender, headers, internet IDs, and conversation topic.
- Body extraction checks Unicode and String8 aliases for text and HTML-string body properties.
- Property-context reports include sorted unknown property-tag IDs for follow-up parser work.

## Public fixture result

The checked-in public PST fixture did not gain selected properties from the String8 alias expansion. The true message candidate still reports 0 selected properties, 74 unknown properties, 0 body payload records, and 1 body fallback row.

This is useful negative evidence: String8 alias support is now present, but the public fixture's selected-property gap is elsewhere.

## Explicit non-goals

- Attachment payload expansion.
- Recipient expansion.
- Full table-row membership decoding.
- Downstream Snowflake, UI, search, or analytics work.

## Next milestone

PQ8 should focus on lower-level property-context layout/tag interpretation. Body, attachment, and recipient expansion should wait until useful properties from the true public-fixture message candidate are selected.
