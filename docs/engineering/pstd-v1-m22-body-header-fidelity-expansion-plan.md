# PSTD v1 M22 Implementation Plan

## Implementation intent

M22 reduces body/header fidelity gaps that are already reachable through the selected MAPI property path. It avoids broad parser work and focuses on deterministic output-contract improvements.

## Current foundation

- Message bodies are emitted as `BodyRecord` rows plus raw body files.
- Binary HTML bodies use `PR_HTML` (`0x1013_0102`).
- Text bodies use `PR_BODY`.
- Transport headers are already selected as `PR_TRANSPORT_MESSAGE_HEADERS`, but were not surfaced on `MessageRecord`.

## Implemented M22 slice

1. Unicode HTML body support:
   - Adds `PR_HTML_STRING` (`0x1013_001f`) to selected MAPI properties.
   - Extracts Unicode HTML bodies into the same deterministic `bodies/<message_key>.html` archive path.
   - Marks Unicode-derived HTML body payloads as UTF-8 encoded after conversion to bytes.
   - Keeps binary HTML as the preferred source when both binary and Unicode HTML properties are present.
2. Header fidelity:
   - Adds `transport_message_headers: Option<String>` to `MessageRecord`.
   - Populates the field from `PR_TRANSPORT_MESSAGE_HEADERS` when present.
   - Leaves status rows and unavailable candidate rows with `None` headers.
3. Regression coverage:
   - Tests Unicode HTML payload creation.
   - Tests Unicode HTML extraction from properties.
   - Tests binary HTML precedence over Unicode HTML.
   - Tests transport header population and absent-header status rows.

## Compatibility rules

| Input shape | Output behaviour |
|---|---|
| `PR_HTML` binary body only | Emit `html` body payload with binary bytes and no encoding label. |
| `PR_HTML_STRING` Unicode body only | Emit `html` body payload with UTF-8 bytes and `encoding=utf-8`. |
| Both binary and Unicode HTML bodies | Prefer binary `PR_HTML` to avoid replacing the byte-faithful source. |
| Missing body properties | Preserve explicit unavailable body status. |
| Missing transport headers | Keep `transport_message_headers=None`. |

## Definition of done

- M22 branch keeps CI green.
- Message output contract documents `transport_message_headers`.
- README, roadmap, PRD, project status, and changelog reflect M22 completion after merge.
- M23 remains focused on attachment payload fidelity.
