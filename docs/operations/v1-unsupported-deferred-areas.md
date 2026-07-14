# PSTD Unsupported and Deferred Areas

_Last reviewed: 14 July 2026._

## Purpose

Record current extraction limitations and intentionally deferred systems. These are explicit boundaries, not capabilities implied by the completed milestone count.

## Current extraction limitations

| Area | Current handling |
|---|---|
| General PST compatibility | Not established. Current claims are based on bounded unit tests and a small approved public fixture. |
| ANSI PST variants | Incomplete and not a supported coverage claim. |
| Uncommon or corrupt layouts | Bounded handling and explicit failures exist, but coverage is not exhaustive. |
| Complete message metadata | Partial. Core metadata must be evaluated field by field from fixture output. |
| Recipient publication | Roles, names, addresses, and address kinds are validated; complete records are not yet emitted from one production fixture execution. |
| SMTP resolution | Native email-address values remain classified as native unless authoritative SMTP properties are present. No heuristic conversion is allowed. |
| Body formats and encodings | Text and RTF evidence is recovered on the public fixture; broad HTML/RTF/encoding fidelity is not established. |
| Attachments | Metadata/parser foundations exist, but the public fixture currently emits zero attachments. Reliable payload extraction is incomplete. |
| Embedded-message attachments | Deep extraction is deferred until the underlying attachment and message subnode paths are validated. |
| Full MAPI property dump | Not implemented as a general feature. The parser supports an evidence-backed selected subset. |
| Exact-preservation archive | Not implemented. Structured extraction is not equivalent to byte-for-byte or MIME-preserving archival. |
| EML generation | Not implemented. It depends on reliable metadata, recipients, bodies, headers, and attachments. |
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

1. complete same-run recipient projection and production publication;
2. inspect the resulting public fixture artifact;
3. select the next highest-value missing email component;
4. expand the approved fixture corpus while preserving fail-closed behaviour.

## Non-negotiable boundaries

- Do not label unsupported data as successfully extracted.
- Do not guess property meaning, address type, encoding, or row alignment.
- Do not combine values from separate executions into one claimed record.
- Do not use downstream systems to hide parser incompleteness.
- Do not treat one public fixture as representative of the full PST ecosystem.
- Do not commit private PST data or expose payloads in CI diagnostics.

## Release language

PSTD may be described as a developing local/Docker PST extractor with substantial bounded parser foundations and validated extraction of selected fields. It should not be described as a complete PST converter, a reliable PST-to-EML tool, or broadly production-ready until representative corpus results support those claims.
