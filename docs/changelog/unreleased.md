# Unreleased

_Last reviewed: 14 July 2026._

## Added

### Product foundation

- Rust `pstd` CLI with `inspect`, `extract`, `batch`, and `version` commands.
- Python operator wrapper and Docker packaging.
- Structured TAR/JSONL output, stable identifiers, run summaries, progress logs, batch checkpoints, resume-by-skip behaviour, and operator handoff documentation.
- Folder, message, body, recipient, threading, attachment, selected-property, manifest, error, and summary record foundations.

### Parser and extraction

- Safe PST header/root selection, bounded byte reads, checked arithmetic, BBT/NBT traversal, block and subnode access, depth/cycle guards, Heap-on-Node, BTH, Property Context, and Table Context parsing.
- Public PST progress workflow and deterministic bounded artifacts.
- Table Context descriptor evidence, bitmap mapping, row-payload candidate resolution, direct/ordinal addressing, validated row transport, fixed-width scalar decoding, and production diagnostics through PQ74.
- Recipient extraction verticals through Vertical 13:
  - semantic `PidTagRecipientType` values;
  - recipient identity HNID extraction;
  - heap-resident `PT_UNICODE` and `PT_STRING8` decoding;
  - end-to-end recipient identity projection;
  - bounded production diagnostics;
  - row-aligned role/name records;
  - SMTP/native/display-name value classification;
  - complete records retaining role, display name, address, and authoritative address kind.

## Changed

- Shifted the active development model from milestone/PQ infrastructure work to evidence-led vertical extraction milestones.
- Prioritised extraction correctness and observable email fields over downstream Snowflake, UI, search, analytics, or graph implementation.
- Corrected B-tree page metadata and child-reference traversal.
- Decoded permitted `NDB_CRYPT_PERMUTE` blocks while preserving internal blocks as raw.
- Tightened payload admission so structurally invalid table declarations fail closed.
- Increased selected public-fixture properties from 0 to 16 and reduced unknown properties from 74 to 19.
- Recovered public-fixture text and RTF body payloads and eliminated the former fallback body row.
- Replaced legacy table assumptions with the real TC heap, row-index BTH, subnode-backed row storage, and four validated 52-byte rows.
- Prevented internal LTP row bookkeeping properties from being reported as user-readable fields.
- Preferred authoritative SMTP/native address properties over display-name fallback while retaining display names separately at the complete-record boundary.
- Rebuilt the root README and current-state documentation so historical milestone/PQ files are no longer presented as the live roadmap.

## Current public-fixture result

```text
BBT/NBT entries: 50/63
Folders: 11
True/extracted messages: 1/1
Body payloads: 2
Attachments: 0
Selected/unknown properties: 16/19
Validated Table Context rows: 4 x 52 bytes
Recipient roles: to, to, cc, cc
Recipient display names: Recipient 1..4
Recipient addresses: to1@domain.com, to2@domain.com, cc1@domain.com, cc2@domain.com
Address classification: native_email_address
```

Complete recipient records can be assembled on `main` from validated role, name, and address diagnostics. Same-run projection and production publication remain incomplete.

## In progress

- Draft PR #430 projects complete recipient records from the same validated rows and heap in one invocation. It is not part of the merged baseline until its exact head passes CI and merges.

## Known limitations

- PSTD is not yet a generally compatible PST converter or PST-to-EML tool.
- Public-fixture attachment output remains zero.
- Recipient complete-record publication is not yet wired through the production fixture run.
- ANSI, uncommon, corrupt, embedded-message, and broad MAPI-layout coverage remain incomplete.
- Downstream Snowflake, UI, search, semantic search, graph, and LLM/RAG work remains parked.

## Removed or superseded

- The earlier assumption that completing M1-M25 made the extraction engine release-complete.
- The PQ-cycle roadmap as the default operating model after the validated parser foundation reached PQ74.
- Stale documentation that described M1-M3 scaffolds or PQ37/PQ57 as the current repository state.
