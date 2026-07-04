# PSTD v1 MVP PRD

## Status

M1-M21 are implemented through milestone branches and intended for CI validation before merge. Four bounded v1 milestones remain after M21: M22-M25. This document defines the local/Docker v1 MVP only.

## Problem

PSTD needs to extract email data from PST files quickly and reliably without using third-party PST parsers such as `libpff`, `pypff`, `readpst`, Outlook COM automation, or other PST extraction libraries. The first implementation must prioritise fast extraction and archive creation while preserving enough metadata, body content, attachments, diagnostics, and stable IDs for future Snowflake search, semantic search, tagging, graph construction, and web review.

## Users

- Primary operator: developer or data engineer running PSTD locally or inside Docker.
- Future operator: Snowpark Container Services worker or batch job orchestrator.
- Future end user: person searching, reviewing, tagging, and downloading extracted email records from a web interface.

## Desired outcome

A v1 local/Docker extraction tool that can process one or many PST files and emit structured TAR archives containing:

- Message metadata.
- Recipient records.
- Threading/reference metadata.
- Text and HTML bodies.
- Extracted attachment files.
- Folder inventory.
- Manifest records.
- Structured errors.
- Summary statistics.
- Progress logs.
- Stable IDs for future table joins and tagging.

## Primary success metric

Speed of extracting and archiving emails from PST files.

## Secondary success metrics

- Completeness of extracted metadata, bodies, and attachments.
- Ability to recover as many emails as possible from corrupt PST files.
- Clear diagnostics when extraction is incomplete.
- Deterministic, Snowflake-ready output contracts.
- Operator visibility through progress logs and summaries.

## Input assumptions

- Typical PST size: 5-10 GB.
- Normal maximum PST size: around 10 GB.
- Normal run size: 20-100 PST files.
- Extreme run size: up to 50,000 PST files in future orchestration scenarios.
- Typical email count: 5,000-10,000 messages per PST.
- Potential total run volume: millions of emails.
- Attachments are common and usually under 5 MB.
- Common attachment types include PDF, Word, Excel, text, images, attached emails, and web pages.
- Most PSTs originate from Exchange, with some from Google-derived exports or migrations.
- Some addresses may appear in Exchange/X.400 format and should be preserved raw and resolved to SMTP where possible.

## MVP scope

### In scope

- Rust-based PST parser and extraction engine built from scratch.
- Python orchestration wrapper for local and Docker execution.
- TAR shard output.
- Structured JSONL metadata files.
- Body and attachment extraction.
- Stable join keys.
- Folder inventory with item counts and type counts where possible.
- Structured error model and continue-on-error behaviour.
- Batch processing across multiple PST files.
- PST-level checkpointing.
- Live console progress and JSONL progress logs.
- Deferred local testing documentation.

### Remaining v1 milestone coverage

| Milestone | Tracking issue | PRD risk reduced |
|---|---:|---|
| M22: Body and Header Fidelity Expansion | #137 | Reduces body/header completeness and status-reporting gaps. |
| M23: Attachment Payload Fidelity | #138 | Reduces attachment payload completeness and unsupported-layout ambiguity. |
| M24: Batch Scale, Performance, and Corruption Hardening | #139 | Reduces operational risk for local/Docker batch runs and recoverable failures. |
| M25: v1 Release Candidate and Operator Handoff | #141 | Confirms validation, documentation, and operator handoff for v1. |

### Out of scope for v1

- Snowflake ingestion.
- Snowpark Container Services deployment.
- React, Vite, Bun, or web UI.
- Keyword search implementation.
- Semantic search implementation.
- Embeddings.
- Knowledge graph construction.
- Email tagging UI or storage.
- Byte-for-byte legal/archive preservation.
- Using external PST parsing libraries.
- Secrets, billing, deployment, production access, or destructive data behaviour.

## Output strategy

PSTD v1 should not use EML as the default canonical output. The default output is structured TAR + JSONL + body files + attachment files.

EML generation may be added later as an optional compatibility or download reconstruction feature, but the v1 archive contract should avoid converting PST to EML only to parse EML again for Snowflake later.

## Required metadata fidelity

Capture where available:

- Subject.
- Sender name.
- Sender email.
- Sender raw address.
- Sender address type.
- Reply-to.
- To, CC, BCC recipients.
- Sent timestamp.
- Received timestamp.
- Created timestamp.
- Modified timestamp.
- Folder path.
- Importance or priority if available.
- Read/unread status if available.
- Attachment count.
- Message size if available.
- Internet Message-ID.
- In-Reply-To.
- References.
- Conversation-Index.
- Conversation-Topic.
- Normalized subject.
- Raw transport headers when available.
- Selected MAPI properties.

## Required body fidelity

- Preserve plain text body when available.
- Preserve HTML body when available.
- Preserve body encoding, size, hash, and status.
- Do not invent missing body representations.
- Record body extraction failures in structured errors.

## Required attachment fidelity

- Extract attachment bytes as raw files inside TAR.
- Preserve original filename in metadata.
- Write safe deterministic archive filename.
- Capture content type, extension, size, hash, inline flag, content ID, attachment order, and extraction status where available.
- Preserve and identify attached emails where possible.
- Record failed attachments without failing the whole message when recoverable.

## Corruption behaviour

Continue on error by default. A corrupt message, folder, attachment, or body should not stop unrelated recoverable extraction. Final statuses must distinguish:

- `success`
- `partial_success`
- `failed`
- `skipped_unsupported_type`
- `skipped_corrupt`

## Future context

Future PSTD phases may load outputs into Snowflake for keyword search, semantic search, tagging, web review, generated email downloads, and possible knowledge graph or LLM/RAG workflows. V1 must preserve stable IDs and output contracts so those later phases do not need to reparse raw PST files.

## Resolved and remaining product questions

- The v1 Rust crate and binary name is `pstd`.
- EML reconstruction remains deferred from the canonical v1 output path; structured TAR + JSONL + body files + attachment files remains the v1 contract.
- ANSI PST support should remain explicitly statused as unsupported until evidence requires a focused milestone.
- Full MAPI property dump mode remains optional audit/debug work unless a later fixture-backed issue proves it is needed for v1 fidelity.
