# PSTD Project Status

_Last reviewed: 14 July 2026._

## Purpose

Provide the single authoritative view of what PSTD can do on `main`, what has been validated against the public fixture, what is only under review, and what remains blocked.

## Current implementation state

| Area | Status on `main` | Evidence and limitations |
|---|---|---|
| Rust CLI and structured output | M1-M25 complete | `inspect`, `extract`, `batch`, and `version`; TAR/JSONL output; run/batch summaries; resume support; Python wrapper; Docker packaging; operator documentation. |
| Header, BBT, NBT, blocks, and subnodes | Validated foundation | Bounded reads, page/block trailers, corrected B-tree metadata and child traversal, cycle/depth limits, and logical node/subnode access. |
| Folder and message discovery | Validated on public fixture | 11 folders and one true message candidate. |
| Property and body extraction | Materially improved | 16 selected properties, 19 unknown properties, text and RTF body payloads recovered, and false table declarations rejected. Coverage remains fixture-limited. |
| Table Context structure | Validated through PQ74 | Real TC heap, row-index BTH, subnode-backed row payload, four 52-byte rows, descriptor mapping, bounded row transport, supported fixed-width scalar decoding, and production diagnostics. |
| Recipient extraction | Validated through Vertical 13 / PR #429 | Four recipient roles, four display names, four native email addresses, authoritative address-kind classification, and fail-closed complete row-aligned record assembly. |
| Same-run complete recipient projection | Not merged | Draft PR #430 projects names and addresses independently from the same rows and heap. Until merged and green, it is not part of the supported baseline. |
| Production publication of complete recipient records | Not complete | `TcHeapDiagnostic` does not yet emit all complete records from one public-fixture execution. |
| Attachments | Incomplete | The public fixture currently emits zero attachments. Existing attachment metadata and parser foundations do not establish reliable attachment coverage. |
| EML reconstruction | Not implemented | Structured TAR + JSONL remains canonical. |
| Downstream systems | Parked | Snowflake, UI, search, analytics, semantic search, and graph work remain out of scope until conversion fidelity is broader. |

## Public-fixture baseline

| Metric | Current stable result |
|---|---:|
| BBT entries | 50 |
| NBT entries | 63 |
| Folders | 11 |
| True message candidates | 1 |
| Extracted messages | 1 |
| Body payloads | 2 |
| Attachments emitted | 0 |
| Selected properties | 16 |
| Unknown properties | 19 |
| Validated Table Context rows | 4 × 52 bytes |

Recipient evidence currently validated from the fixture:

| Row | Role | Display name | Address | Address kind |
|---:|---|---|---|---|
| 0 | To | Recipient 1 | `to1@domain.com` | native email address |
| 1 | To | Recipient 2 | `to2@domain.com` | native email address |
| 2 | Cc | Recipient 3 | `cc1@domain.com` | native email address |
| 3 | Cc | Recipient 4 | `cc2@domain.com` | native email address |

These values were validated through bounded Table Context, row, HNID, Heap-on-Node, encoding, and semantic-property checks. On `main`, complete records can be assembled only when the validated role, name, and address diagnostics are supplied together. They are not yet published by one production fixture run.

## Progress sequence

| Range | Result |
|---|---|
| M1-M25 | Built the complete local/Docker product foundation and structured output lane. |
| PQ1-PQ5 | Corrected root/index traversal and real folder/message classification. |
| PQ6-PQ35 | Expanded property/body measurement, Heap-on-Node/BTH support, payload/subnode diagnosis, and replaced invalid table assumptions. |
| PQ36 | Decoded permitted blocks, tightened payload admission, recovered body data, and materially reduced unknown-property output. |
| PQ37-PQ57 | Resolved the real Table Context structures and validated four bounded row layouts and masks. |
| PQ58-PQ74 | Validated descriptor mapping, row addressing/transport, fixed-width scalar decoding, and bounded production diagnostics. |
| Vertical 1-3 | Classified real row properties and exposed recipient types. |
| Vertical 4-8 | Extracted recipient string references, resolved heap strings, built an end-to-end projection, and integrated recipient identity diagnostics into production reporting. |
| Vertical 9-10 | Assembled and published role/name recipient records. |
| Vertical 11-12 | Preferred address-bearing properties and classified native versus SMTP address output. |
| Vertical 13 | Added complete record assembly retaining role, display name, address, and address kind by row. |

## Active work

Draft PR #430, **Project complete recipient records in one run**, is the current implementation lane. Its objective is to project `PidTagDisplayName` and the preferred address property independently from the same validated rows and heap, then assemble complete records without combining evidence from separate executions.

At the time of this review, the PR is draft and its latest CI run failed. The failure must be diagnosed and the exact head must pass the full workflow before the capability is described as merged.

## Next evidence-based milestones

1. Repair and validate PR #430 so the complete-recipient projection works from one row/heap execution.
2. Integrate that projection into production Table Context reporting and publish bounded complete-recipient records from the public fixture.
3. Re-run the fixture and choose the next highest-value extraction gap from measured output rather than extending parser infrastructure automatically.
4. Prioritise gaps that move one real message closer to reconstructable email output, especially missing core metadata, unsupported body forms, and attachment payloads.

The later ordering is intentionally provisional. Fixture evidence after each vertical milestone must be allowed to change the plan.

## Validation expectations

Every extraction PR must pass formatting, clippy with warnings denied, all Rust tests, CLI smoke checks, Python wrapper checks, Docker build, approved fixture inspect/extract/batch checks, deterministic public-PST artifact generation, and review of the exact artifact delta.

## Risk statement

PSTD now has substantial bounded parser coverage and concrete recipient extraction behaviour, but it remains a developing extractor rather than a generally compatible converter. One small public fixture cannot establish support for the range of ANSI/Unicode PST versions, corruption patterns, property encodings, embedded messages, attachment layouts, and uncommon MAPI structures found in real archives.
