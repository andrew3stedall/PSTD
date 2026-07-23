# PSTD Roadmap

_Last reviewed: 23 July 2026._

## Objective

Deliver reliable, evidence-backed email extraction from Microsoft Purview Unicode PST exports before investing in downstream systems or lower-value format breadth. Progress is measured by new correct observable extraction and EML behaviour on approved fixtures while preserving bounded, deterministic, fail-closed behaviour.

## Roadmap principles

- Prioritise end-to-end extraction capability over parser infrastructure for its own sake.
- Treat Microsoft Purview Unicode exports as the primary producer target.
- Implement the smallest coherent vertical slice that exposes new behaviour.
- Prefer representative corpus breadth over abstractions that do not change observable extraction.
- Fail closed when bounds, types, references, ownership, encodings, recursion, or completeness do not validate.
- Preserve existing stable identifiers and deterministic fixture output unless an intentional contract change is documented.
- Re-run every approved fixture after each milestone and revise the next milestone from measured artifacts.
- Treat contacts, appointments, tasks, journals, and distribution lists as typed non-mail objects rather than forcing them into EML.
- Keep PSTD self-contained. External PST implementations may support isolated fixture generation and comparison but cannot become required shipped dependencies.
- Keep Snowflake, UI, search, analytics, semantic search, and graph work parked.

## Completed foundation

### Product and parser foundation

M1-M25 and PQ1-PQ74 are complete. The repository has a Rust CLI, Python wrapper, Docker packaging, structured TAR/JSONL output, stable IDs, batch/resume, diagnostics, bounded PST traversal, Heap-on-Node/BTH/Property Context/Table Context decoding, row transport, and selected MAPI values.

### Original fixture email path

The approved original fixture emits four exact To/Cc recipients, validated plain text and HTML recovered from RTF, and one deterministic 956-byte `multipart/alternative` EML.

### Tika attachment and embedded-message path

Complete through Vertical 38:

- eight messages including one separately linked method-5 child;
- nine directly owned recipient records;
- eight exact folders and seven exact top-level physical message owners;
- six valid body payloads totalling 271 bytes and two explicit unavailable HTML forms;
- one exact 11,862-byte DOCX by-value attachment;
- one deterministic 17,035-byte parent `multipart/mixed` EML;
- one deterministic 453-byte child `text/plain` EML;
- one byte-identical 453-byte method-5 `message/rfc822` payload;
- fail-closed rejection of missing, mismatched, duplicate, nested, ambiguous, and unsafe child candidates.

### ANSI diagnostics

Vertical 39 decodes version-14/15 header roots and crypt-method fields for diagnostics only. ANSI page traversal and email extraction remain unsupported. This diagnostic lane does not establish user-visible email compatibility.

### Comparison corpus baseline

PR #491 records the java-libpst fixture's deterministic fail-closed result: 25 folders, 9 message metadata records, 12 body records, 0 recipients, 22 attachment metadata records, 0 materialised payloads, 0 validated `IPM.Note*` classes, and 0 EML files.

## Current milestone

### First controlled Microsoft Purview Unicode export

Admit the smallest redistributable synthetic Purview export that exposes a capability not already proven by current fixtures. The first preference is multiple by-value attachments with exact ownership. If the first approved export instead provides verified inline CID evidence, authoritative Exchange-to-SMTP mapping, another embedded-message layout, or broader HTML/RTF evidence, select the smallest coherent vertical supported by those bytes.

Fixture admission requires:

1. a synthetic source-mailbox manifest containing no private data;
2. a documented Purview export procedure and relevant settings;
3. explicit redistribution basis or reproducible controlled generation;
4. immutable PST bytes, exact length, and SHA-256;
5. header version and encryption classification;
6. an independent inventory from at least one pinned comparison implementation;
7. two PSTD baseline runs with byte-identical semantic output;
8. exact folder, message, recipient, body, attachment, typed-object, diagnostic, and EML counts;
9. exact materialised payload and EML paths, lengths, hashes, ownership, and MIME structure;
10. explicit unavailable, unsupported, ambiguous, corrupt, encrypted, or incomplete statuses.

The detailed corpus plan is `docs/operations/purview-unicode-corpus-plan.md`.

## Purview capability sequence

After the first admitted baseline, prioritise the smallest evidenced incomplete capability in this order:

1. multiple and broader by-value attachment layouts, formats, filenames, and sizes;
2. inline attachments and exact Content-ID/HTML `cid:` correlation;
3. complete multi-folder and multi-message discovery across representative exports;
4. independent plain, HTML, and RTF body forms and encodings;
5. authoritative Exchange-to-SMTP resolution while preserving unresolved native evidence;
6. additional method-5 layouts and bounded nested embedded-message recursion;
7. typed non-mail classification and explicit completeness status;
8. corrupt, truncated, duplicate, cross-scope, encrypted, and ambiguous cases;
9. large-export performance and memory hardening;
10. a narrow stable Rust API after extraction records and diagnostics stabilise.

ANSI traversal remains a later compatibility lane. A deterministic empty ANSI container is useful structural evidence but adds no observable email or EML behaviour and therefore does not outrank Purview Unicode coverage.

## Fixture and dependency boundary

Fixtures may originate from controlled Purview exports, public repositories, or controlled synthetic generation when provenance or generation recipe, redistribution basis, immutable revision or pinned tool version, exact path, byte length, and SHA-256 are documented.

Do not add Outlook, libpff, libpst, java-libpst, Apache Tika, or another PST parser/converter as a required library, build, runtime, normal test-runtime, Docker, Python-wrapper, or end-user dependency. Pinned tools may run in isolated fixture-generation or comparison workflows. PSTD acceptance must be established separately by its own Rust implementation and exact fixture evidence.

Agreement with another implementation is supporting evidence rather than authoritative truth. Parser rules must not be relaxed merely to match another tool. Unsupported, encrypted, malformed, duplicate, ambiguous, out-of-range, or unowned structures remain explicit and emit no guessed EML.

## Stable Rust API boundary

The eventual public API should support:

- opening a PST with explicit parser limits;
- iterating typed mail and non-mail objects;
- reading message, recipient, body, and attachment records;
- generating deterministic EML from admitted evidence;
- reading diagnostics and completeness statuses;
- distinguishing unavailable, unsupported, corrupt, encrypted, and ambiguous evidence.

Internal page, heap, BTH, and Table Context implementations must not become the compatibility contract.

## Completion definition

PSTD must not be described as generally reliable for Purview or production-ready until a representative controlled Purview corpus demonstrates, with exact completeness statuses:

- folder hierarchy and message discovery without silent omissions;
- core headers, dates, identifiers, and transport evidence;
- To/Cc/Bcc recipients and authoritative addresses where available;
- plain text, HTML, and RTF handling across encodings;
- by-value, inline, embedded, and nested attachments within explicit limits;
- typed non-mail classification;
- deterministic structured output and EML assembly;
- fail-closed malformed, encrypted, corrupt, unsupported, and ambiguous behaviour;
- no regressions across all approved fixtures;
- documented performance and memory limits.

## Deferred roadmap

1. ANSI root traversal and message extraction after Purview Unicode coverage is materially broader.
2. Snowflake ingestion.
3. Search and review web application.
4. Semantic search, embeddings, tagging, graph, and LLM/RAG workflows.
5. Distributed orchestration beyond the current local/Docker batch model.