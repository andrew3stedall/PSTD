# PSTD Product Requirements

_Last reviewed: 14 July 2026._

## Status

The M1-M25 product-foundation lane is complete, but the product outcome is not complete. Active development is focused on extraction fidelity through small vertical milestones. PSTD is not yet a broadly reliable PST-to-email or PST-to-EML converter.

## Problem

PSTD must extract email data from Microsoft Outlook PST files quickly, correctly, and transparently without relying on external PST parsing libraries or Outlook automation. It must preserve enough metadata, recipients, bodies, threading information, attachments, folder context, and diagnostics to support later EML assembly and downstream loading.

The main risk is not missing infrastructure. It is silently producing incomplete or incorrectly interpreted email data from complex PST structures.

## Users

- Primary operator: developer or data engineer running PSTD locally or in Docker.
- Future operator: automated batch worker after local extraction is proven.
- Future end user: person searching, reviewing, tagging, or downloading reconstructed email data.

## Desired outcome

A local/Docker extraction tool that can process one or many PST files and produce deterministic structured archives containing, where present and validated:

- folder hierarchy and inventory;
- message identity and metadata;
- sender and recipient records;
- threading/reference data;
- text, HTML, and RTF-derived body representations;
- attachment metadata and raw payloads;
- selected MAPI evidence needed for audit and completeness;
- manifests, structured warnings/errors, summaries, progress, and stable IDs.

## Success criteria

Correctness and completeness take precedence over throughput until extraction fidelity is reliable.

Primary measures:

1. percentage of real messages discovered without false positives;
2. completeness of required metadata, recipients, bodies, and attachments;
3. explicit unsupported/partial statuses instead of silent omission or guessing;
4. deterministic output and no regressions across the approved fixture corpus;
5. safe continuation around recoverable corruption;
6. throughput and resource use once correctness requirements are met.

## Current validated baseline

The public fixture currently produces:

- 50 BBT entries and 63 NBT entries;
- 11 folders;
- one true and extracted message;
- two body payloads;
- zero attachments;
- 16 selected and 19 unknown properties;
- four validated 52-byte Table Context rows;
- two To and two Cc recipient roles;
- four display names and four native email-address values;
- complete row-aligned recipient assembly at the library boundary.

Same-run complete recipient projection and production publication remain incomplete.

## In scope

- Rust PST parser and extraction engine built from repository-owned code.
- Thin Python operator wrapper.
- Local and Docker execution.
- Single-PST and batch processing.
- Bounded PST traversal and selected MAPI decoding.
- Structured TAR/JSONL output.
- Folder, message, recipient, body, threading, and attachment extraction.
- Stable identifiers and explicit completeness statuses.
- Continue-on-error and deterministic batch resume/skip behaviour.
- Synthetic regression fixtures and approved public/sanitised integration fixtures.
- Public progress artifacts and evidence-led roadmap revision.
- Later optional EML assembly after source extraction is sufficiently complete.

## Out of scope until extraction is reliable

- Snowflake ingestion and Snowpark deployment.
- React/Vite or other web UI.
- Keyword or semantic search.
- Embeddings, graph construction, tagging, and LLM/RAG workflows.
- Distributed orchestration beyond measured need.
- Secrets, billing, production access, or destructive source behaviour.
- External PST parser libraries or Outlook COM automation.

## Required metadata fidelity

Capture when present and authoritative:

- subject and normalized subject;
- sender name, raw address, address type, and SMTP address when proven;
- reply-to and To/Cc/Bcc recipients with display names and usable addresses;
- sent, received, created, and modified timestamps;
- folder path and source node identity;
- importance, priority, read state, message size, and attachment count;
- Internet Message-ID, In-Reply-To, References, Conversation-Index, and Conversation-Topic;
- transport headers;
- selected MAPI property identity, type, source, and completeness status.

Do not infer SMTP from native address values, combine values from separate runs, or publish partial row-aligned records.

## Required body fidelity

- Preserve plain text when present.
- Preserve HTML when present, including validated binary or string property forms.
- Handle RTF-derived content according to an explicit fidelity policy.
- Preserve encoding, source property, size, hash, and status.
- Do not invent a missing body representation.
- Record unavailable, unsupported, or malformed bodies explicitly.

## Required attachment fidelity

- Preserve metadata for every validated attachment row.
- Extract raw bytes into TAR entries when available.
- Preserve original and safe filenames, media type, extension, size, declared size, hash, inline state, content ID, method, ordinal, archive path, and status.
- Retain metadata-only rows for unavailable, empty, unsupported, or deferred payloads.
- Treat embedded messages explicitly rather than as ordinary opaque files when evidence permits.
- Avoid failing unrelated messages for recoverable attachment defects.

## Required batch fidelity

- Keep discovered, attempted, completed, partial, failed, skipped, and not-run PST counts separate.
- Write append-only checkpoint and progress streams.
- Make resume-by-skip deterministic unless overwrite is requested.
- Preserve per-PST output and status even when a later PST fails.
- Support large PST files without loading entire archives into memory.

## Corruption and ambiguity behaviour

- Use bounded reads and checked arithmetic.
- Apply depth, count, allocation, and cycle limits.
- Continue around recoverable local defects.
- Fail closed when candidate selection, property identity, row mapping, reference type, encoding, or payload bounds are ambiguous.
- Never weaken validation solely to recover a previous counter.
- Distinguish `success`, `partial_success`, `failed`, `missing`, `unsupported`, `unavailable`, `skipped`, and stopped-early states.

## Output strategy

The canonical output remains structured TAR + JSONL plus raw body and attachment files. EML is a future assembly/download format, not the parser’s primary internal representation. Exact-preservation requirements may require richer raw evidence than the current output contract.

## Delivery model

- M1-M25 established the product and operating foundation.
- PQ1-PQ74 established the validated parser and Table Context foundation.
- Current work uses evidence-led vertical extraction milestones.
- Every milestone must add observable extraction value, preserve existing behaviour, pass full CI, rerun the public fixture, and revise the next milestone from measured output.

## Product completion boundary

The product is not complete until a representative fixture corpus demonstrates reliable folder, message, metadata, recipient, body, threading, and attachment extraction with explicit completeness states and deterministic outputs sufficient for the chosen EML or downstream fidelity target.
