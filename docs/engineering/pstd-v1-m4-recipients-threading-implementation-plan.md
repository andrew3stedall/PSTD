# PSTD v1 M4 Implementation Plan

## Implementation intent

M4 turns PSTD's message metadata layer into a conversation-aware metadata extractor. The first implementation slice should be conservative: emit the M4 output files, add property constants and helper logic, and only populate fields when the parser has sufficient source data.

## Current foundation

- `MessageRecord` already contains threading fields such as `internet_message_id`, `in_reply_to_id`, `conversation_index`, `conversation_topic`, and `normalized_subject`.
- `RecipientRecord` and `MessageReferenceRecord` structs already exist in `src/output/metadata.rs`.
- Existing archive output writes `folders.jsonl`, `messages.jsonl`, and M3 internal metadata files.

## Initial code changes

1. Add selected MAPI constants for:
   - Internet message ID.
   - In-Reply-To ID.
   - Internet References.
   - Transport headers.
   - Conversation index.
   - Conversation topic.
   - Sender raw and address-type fields.
   - Recipient display name, address type, email address, SMTP address, and recipient type.
2. Add a `pst::threading` helper module for:
   - Normalized subject calculation.
   - Prefix removal for common reply/forward prefixes.
   - Internet references splitting.
   - Threading status calculation.
3. Extend `MetadataExtractionOutput` with recipient and message-reference rows.
4. Write `data/recipients.jsonl` and `data/message_references.jsonl` into TAR output.
5. Add manifest entries for the new files.

## Later M4 parser work

- Parse recipient table contexts once table traversal can locate recipient rows reliably.
- Resolve SMTP values from direct SMTP fields first, then preserve Exchange/X.400 raw address data.
- Preserve raw transport headers when available, but avoid deriving false precision from partial headers.
- Add fixture-backed tests when safe PST fixtures are available.

## Safety and privacy

Do not commit private PST files. Any fixture added to the repo must be synthetic, explicitly public, or non-sensitive and license-compatible.

## Definition of done

- M4 branch keeps CI green.
- New M4 output files are emitted.
- Threading helpers are covered by unit tests.
- Missing or unsupported M4 fields are represented with explicit status values.
- M5 body/attachment work remains out of scope.
