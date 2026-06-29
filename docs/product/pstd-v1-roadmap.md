# PSTD v1 Roadmap

## Roadmap principles

- Build v1 as a local/Docker Rust + Python extraction tool.
- Keep Snowflake, frontend, and deployment concerns out of v1 implementation.
- Make the output contract future-ready for Snowflake search, semantic search, tagging, review, download, and graph work.
- Work at milestone and epic level.
- Defer local tests when working from the phone/GitHub connector workflow, but record what should be run later.

## Roadmap overview

```text
M1: Extraction Foundation and Archive Contract [implemented, validation deferred]
M2: PST Binary Foundation [implemented, validation deferred]
M3: Folder and Metadata Extraction [implemented on branch, validation deferred]
M4: Recipients, Threading, and Address Resolution [next]
M5: Bodies and Attachments
M6: Batch Orchestration, Checkpointing, and Progress
M7: Performance, Resilience, and Release Candidate
```

## Milestone M1: Extraction Foundation and Archive Contract

Implemented and merged to `main` via PR #18. Issues #7-#16 are closed as completed. Local validation remains deferred.

## Milestone M2: PST Binary Foundation

Implemented and merged to `main` via PR #30. Issues #19-#28 are closed as completed. Local validation remains deferred.

## Milestone M3: Folder and Metadata Extraction

Implemented on branch `pstd-v1-m3-folder-metadata`. Pull request review and local validation are pending.

Delivered:

- Logical node/block access.
- Heap-on-node parsing.
- BTH parsing.
- Property context parsing.
- Table context parsing.
- Selected MAPI property decoding.
- Root folder inventory output.
- Initial message metadata/status output.
- Metadata-only archive output.
- CLI path through `pstd extract --manifest-only`.

Still out of scope:

- Message bodies.
- Attachments.
- Recipients.
- Full threading.
- Snowflake.
- Web UI.

## Milestone M4: Recipients, Threading, and Address Resolution

Next planned milestone.

## Future roadmap after v1

```text
V2: Snowflake ingestion
V3: Search and review web application
V4: Knowledge graph and LLM support
```
