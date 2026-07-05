# PQ7 Selected Property Dictionary Expansion

## Purpose

PQ7 expands selected MAPI property recognition after PQ6 showed that the public PST fixture's true message candidate has a loadable property context, but no parsed properties were mapped into the selected dictionary.

## What changed

PQ7 adds safe recognition for ANSI/String8 (`0x001e`) variants of already selected Unicode string (`0x001f`) properties.

This means PSTD now recognises both forms for selected message, body, recipient, and attachment metadata tags where the property ID is already in scope.

## Safe mapping rule

PQ7 does not invent semantics for unknown property IDs. It only maps String8 variants where the Unicode property ID was already selected.

Examples:

| Unicode tag | String8 tag | Selected name |
|---:|---:|---|
| `0x0037001f` | `0x0037001e` | `subject` |
| `0x007d001f` | `0x007d001e` | `transport_message_headers` |
| `0x1000001f` | `0x1000001e` | `body_text` |
| `0x1013001f` | `0x1013001e` | `body_html_unicode` |

## Diagnostics

`PropertyContextParseReport` now records sorted unknown property-tag IDs in addition to the unknown count. This keeps the next parser expansion grounded in observed tags without logging raw message body or attachment payload data.

## Explicit non-goals

- Attachment payload expansion.
- Recipient expansion.
- Full message-table row membership decoding.
- Mapping unknown property IDs without known MAPI semantics.
- Snowflake, UI, search, or analytics work.

## Next blocker

The next blocker should be selected from the public PST artifact after PQ7 CI completes. If selected property coverage remains low, the next step is likely lower-level property-context layout work rather than broader payload expansion.
