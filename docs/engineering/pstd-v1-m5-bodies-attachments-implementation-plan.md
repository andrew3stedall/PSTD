# PSTD v1 M5 Implementation Plan

## Implementation intent

M5 adds payload-aware archive output while staying conservative about parser depth. The first implementation slice should establish body and attachment records, deterministic archive paths, and helper functions that can be fed by deeper PST traversal later.

## Current foundation

- `BodyRecord` and `AttachmentRecord` already exist in `src/output/metadata.rs`.
- The TAR writer can append arbitrary byte payloads by path.
- M4 already emits metadata, recipients, and message references.
- Current BBT/NBT and table traversal remain skeleton-level, so M5 must represent missing payloads explicitly.

## Initial code changes

1. Add selected MAPI constants for body and attachment fields.
2. Replace body/attachment placeholder modules with helper logic.
3. Extend `MetadataExtractionOutput` with body and attachment records.
4. Emit `data/bodies.jsonl` and `data/attachments.jsonl` even when empty.
5. Add manifest entries for the new output files.
6. Preserve explicit status values for unavailable payloads.

## Later parser work

- Locate body payloads from property contexts once message property traversal is deeper.
- Locate attachment table rows and attachment data blocks once attachment subnode traversal is deeper.
- Preserve raw payload bytes and hash values without text reinterpretation.
- Add synthetic/public fixture cases for text body, HTML body, and attachments.

## Safety and privacy

Do not commit private PST files or extracted private email payloads. Any fixture added to the repo must be synthetic, explicitly public, or non-sensitive and license-compatible.

## Definition of done

- M5 branch keeps CI green.
- New body and attachment output files are emitted.
- Body helper logic is covered by unit tests.
- Attachment helper logic is covered by unit tests.
- Missing or unsupported payload fields are represented with explicit status values.
- M6 batch orchestration remains out of scope.
