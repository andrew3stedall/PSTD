# PQ6 Property and Body Coverage

## Purpose

PQ6 makes message property-context and body coverage measurable for the true message candidate set introduced by PQ5.

PQ5 corrected public fixture message counting by emitting only decoded normal/associated message nodes as message rows. PQ6 asks the next conversion-quality question: for those true message candidates, how much metadata and body content can PSTD actually extract?

## What changed

PQ6 adds explicit coverage counters for:

- message candidates with loaded property contexts;
- message candidates where property context is unavailable;
- selected MAPI properties, unknown properties, and decode errors;
- messages with supported body properties;
- extracted body payload messages and records;
- body fallback records;
- text, HTML, and RTF body-property evidence.

## Body status model

Body extraction remains bounded to already decoded property context values.

| Status | Meaning |
|---|---|
| `body_payload_extracted` | At least one supported body payload was emitted. |
| `body_payload_property_absent` | No supported body property was found. |
| `body_payload_properties_present_but_unusable` | A supported body property existed, but no payload could be emitted from the decoded/raw value. |
| `node_property_context_unavailable` | The message property context could not be loaded, so body extraction was not possible. |

## Extraction status counters

The extraction status includes `pq6_status`, which records:

- `property_loaded_messages`
- `property_unavailable_messages`
- `selected_properties`
- `unknown_properties`
- `decode_errors`
- `body_supported_property_messages`
- `body_payload_messages`
- `body_payload_records`
- `body_fallback_records`
- `text_body_property_messages`
- `html_body_property_messages`
- `rtf_body_property_messages`

A status row is also written to the error/status stream for milestone auditability.

## Known limitations after PQ6

PQ6 improves measurement and fallback evidence. It does not add new low-level property-context layouts, table-row membership decoding, recipient expansion, or attachment payload extraction.

## Next blocker

The next blocker is PQ7: attachment and recipient coverage. PQ6 should leave PSTD with an honest message/body coverage baseline for the public fixture before broadening into attachments and recipients.
