# PSTD v1 M4 Implementation Plan

## Implementation intent

M4 turns PSTD's message metadata layer into a conversation-aware metadata extractor. The implementation remains conservative: emit the M4 output files, add property constants and helper logic, and populate fields only when the parser has sufficient source data.

## Current foundation

- `MessageRecord` contains threading fields such as `internet_message_id`, `in_reply_to_id`, `conversation_index`, `conversation_topic`, and `normalized_subject`.
- `RecipientRecord` and `MessageReferenceRecord` structs exist in `src/output/metadata.rs`.
- Archive output writes `folders.jsonl`, `messages.jsonl`, `recipients.jsonl`, and `message_references.jsonl`.
- `pst::threading` contains deterministic subject/reference helper logic.
- `pst::recipients` converts parsed recipient table rows into recipient output records.

## Implemented M4 slices

1. Selected MAPI constants for:
   - Internet message ID.
   - In-Reply-To ID.
   - Internet References.
   - Transport headers.
   - Conversation index.
   - Conversation topic.
   - Sender raw and address-type fields.
   - Recipient display name, address type, email address, SMTP address, and recipient type.
2. Threading helpers for:
   - Normalized subject calculation.
   - Prefix removal for common reply/forward prefixes.
   - Reference splitting.
   - Threading status calculation.
3. Metadata output wiring for:
   - `data/recipients.jsonl`.
   - `data/message_references.jsonl`.
   - Manifest entries for both files.
4. Recipient conversion for:
   - To, CC, BCC, reply-to, unknown recipient type mapping.
   - Display name extraction.
   - Raw address preservation.
   - SMTP address preference.
   - Exchange/X.400-style raw address preservation when SMTP is unavailable.
   - Stable recipient keys and ordinals.

## Remaining parser depth risk

The recipient conversion layer expects rows that have already been parsed into `TableContext`. Current M2/M3 PST traversal remains skeleton-level, so broader real-world recipient extraction depends on later improvements to BBT/NBT traversal and table-context discovery.

## Safety and privacy

Do not commit private PST files. Any fixture added to the repo must be synthetic, explicitly public, or non-sensitive and license-compatible.

## Definition of done

- M4 branch keeps CI green.
- New M4 output files are emitted.
- Threading helpers are covered by unit tests.
- Recipient row conversion is covered by unit tests.
- Missing or unsupported M4 fields are represented with explicit status values.
- M5 body/attachment work remains out of scope.
