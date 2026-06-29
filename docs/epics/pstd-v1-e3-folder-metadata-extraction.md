# Epic E3: Logical PST Structures and Metadata Extraction

## Outcome

Turn the M2 binary foundation into the first useful metadata extraction layer for real PST files.

## Scope

This epic covers:

- Logical node/block access boundary.
- Heap-on-node parser foundation.
- BTH parser foundation.
- Property context parser.
- Table context parser.
- Selected MAPI property registry and value decoder.
- Folder hierarchy traversal.
- Folder inventory output.
- Initial message metadata output.
- Metadata-only archive writing through the existing structured TAR + JSONL contract.

## Out of scope

- Body extraction.
- Attachment extraction.
- Recipient extraction.
- Full threading extraction.
- Address resolution.
- Search indexes.
- Snowflake loading.
- Web UI.

## System flow

```text
pstd extract --input archive.pst --output ./out --manifest-only
  -> M2 reader/header/index/block layer
  -> logical node access
  -> heap/BTH/property/table contexts
  -> folder traversal
  -> message metadata extraction
  -> structured TAR + JSONL output
```

## Success criteria

- Folder inventory is populated from PST data rather than placeholders.
- Message metadata rows are populated from PST data rather than placeholders.
- Missing unsupported values are recorded explicitly.
- Bodies and attachments remain deferred to later milestones.
- M4 can add recipients/threading without rewriting M3 metadata extraction.

## Risks

| Risk | Rating | Mitigation |
|---|---:|---|
| PST logical structures are more complex than expected | High | Keep M3 focused on metadata-only extraction |
| Fixture availability is limited | High | Use synthetic fixtures plus approved local PST fixture only |
| Property decoding becomes too broad | Medium | Decode selected properties first |
| Table context parsing blocks folder traversal | Medium | Add skeletons and clear unsupported diagnostics |

## Handoff to M4

M4 should consume M3 message records and add recipients, thread/reference fields, conversation metadata, and address resolution.
