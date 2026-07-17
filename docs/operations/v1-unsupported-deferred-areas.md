# PSTD Unsupported and Deferred Areas

_Last reviewed: 17 July 2026._

## Purpose

Record current extraction limitations and intentionally deferred systems. These are explicit boundaries, not capabilities implied by the completed milestone count.

## Current extraction limitations

| Area | Current handling |
|---|---|
| General PST compatibility | Not established. Current claims are based on bounded unit tests and a small approved public fixture. |
| ANSI PST variants | Incomplete and not a supported coverage claim. |
| Uncommon or corrupt layouts | Bounded handling and explicit failures exist, but coverage is not exhaustive. |
| Complete message metadata | Partial. Core metadata must be evaluated field by field from fixture output. |
| Recipient publication | Complete row-aligned records are fixture validated for the original PST and for nine directly owned Tika rows; broad layout coverage remains incomplete. |
| SMTP resolution | Native email-address values remain classified as native unless authoritative SMTP properties are present. No heuristic conversion is allowed. |
| Body formats and encodings | Text, RTF-derived HTML, and raw HTML-property evidence are recovered on approved fixtures; broad HTML/RTF/encoding fidelity is not established. |
| Attachments | One Tika DOCX payload is exact and fixture validated. Other methods, formats, and layouts remain incomplete. |
| Embedded-message attachments | One method-`5` child is emitted separately and linked from its parent. Child EML payload materialisation, nested attachments, ambiguity beyond the fixture, and recursion remain deferred. |
| Full MAPI property dump | Not implemented as a general feature. The parser supports an evidence-backed selected subset. |
| Exact-preservation archive | Not implemented. Structured extraction is not equivalent to byte-for-byte or MIME-preserving archival. |
| EML generation | Fixture-limited plain/HTML and plain/DOCX assembly is validated. Attachmentless plain-text child EML and broad RFC fidelity remain incomplete. |
| Broad fixture corpus | Needed before release-quality compatibility claims. |

## Deferred systems

| Area | Status | Start condition |
|---|---|---|
| Snowflake ingestion | Parked | Extraction output is sufficiently complete and stable to load without institutionalising gaps. |
| Snowpark Container Services | Parked | A justified Snowflake execution model exists after ingestion design. |
| Web UI and review workflow | Parked | Stable storage/query and extraction completeness exist. |
| Keyword and semantic search | Parked | Downstream data is reliable and search requirements are defined. |
| Embeddings and LLM/RAG | Parked | Privacy, model, cost, retrieval, and evaluation decisions are complete. |
| Knowledge graph | Parked | Entity, recipient, threading, and reference extraction are stable. |
| Tagging | Parked | Storage and UI ownership are defined. |
| Distributed orchestration | Parked | Local/Docker batch limits are measured and justify added complexity. |

## Active priority

The current priority is conversion coverage only:

1. emit the recovered Tika child as a deterministic plain-text-only EML;
2. keep its four raw HTML-property bytes out of MIME output;
3. decide method-`5` payload materialisation only under a separate exact contract;
4. expand the approved fixture sequence while preserving fail-closed ownership.

## Non-negotiable boundaries

- Do not label unsupported data as successfully extracted.
- Do not guess property meaning, address type, encoding, or row alignment.
- Do not combine values from separate executions into one claimed record.
- Do not use downstream systems to hide parser incompleteness.
- Do not treat one public fixture as representative of the full PST ecosystem.
- Do not commit private PST data or expose payloads in CI diagnostics.

## Release language

PSTD may be described as a developing local/Docker PST extractor with bounded parser foundations and fixture-validated message, recipient, body, DOCX, embedded-message, and EML paths. It should not be described as a complete PST converter, a generally reliable PST-to-EML tool, or broadly production-ready until representative corpus results support those claims.
