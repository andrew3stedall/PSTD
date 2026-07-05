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

## Explicit non-goals

- Attachment payload expansion.
- Recipient expansion.
- Full table-row membership decoding.
- Downstream Snowflake, UI, search, or analytics work.

## Next milestone

The next milestone should be chosen from the measured PQ7 public fixture result. If selected property coverage remains low, PQ8 should focus on lower-level property-context layout fidelity. If selected coverage improves enough to expose body, attachment, or recipient evidence, PQ8 should target that next measured gap.
