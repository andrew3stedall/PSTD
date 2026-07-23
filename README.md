# PSTD

PSTD is a Rust-first tool for extracting email data from Microsoft Outlook PST files. The immediate objective is reliable, evidence-backed PST-to-EML coverage for Microsoft Purview Unicode exports. Downstream Snowflake, search, UI, analytics, and graph work remain parked until extraction fidelity is substantially broader.

## Current position

_Last reviewed: 23 July 2026._

| Area | State on `main` | Current result |
|---|---|---|
| Product foundation | Complete through M25 | Rust CLI, Python wrapper, Docker packaging, structured TAR/JSONL output, batch/resume support, diagnostics, and operator guidance. |
| Parser-quality sequence | Complete through PQ74 | Bounded PST traversal, Heap-on-Node/BTH/Table Context parsing, validated row transport, fixed-width value decoding, and production diagnostics. |
| Vertical extraction sequence | Complete through Vertical 39 | Four-byte Property Context body locators remain explicit unavailable forms; ANSI version-14/15 header fields are decoded with variant-correct widths but cannot authorize traversal or extraction. |
| Current milestone | First controlled Microsoft Purview Unicode export | Admit immutable, redistributable synthetic Purview bytes, lock the exact before-state, then implement the smallest newly evidenced email-to-EML capability. |
| EML reconstruction | Three deterministic outputs across two fixtures | The original fixture emits one 956-byte plain/HTML EML; Tika emits the unchanged 17,035-byte plain-text/DOCX parent and one exact 453-byte plain-text child. |

## Intent

PSTD is intended to become a dependable PST-to-email extraction engine that:

- preserves folder and message relationships;
- extracts message metadata, bodies, recipients, threading data, and attachments;
- fails closed when a PST structure is unsupported or ambiguous;
- records explicit diagnostics instead of silently guessing;
- produces deterministic structured output suitable for later EML generation or downstream loading;
- validates every material parser change against synthetic tests and approved public or controlled PST fixtures;
- remains self-contained rather than delegating PST parsing or conversion to another implementation.

## Validated fixture evidence

The existing approved fixtures establish two material Unicode paths, but neither is a Microsoft Purview export and neither establishes broad Purview compatibility.

The original public fixture currently yields:

| Metric | Validated result |
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

The Table Context path validates four 52-byte rows. The fixture has separately produced:

- recipient roles: `to`, `to`, `cc`, `cc`;
- display names: `Recipient 1` through `Recipient 4`;
- native email-address values: `to1@domain.com`, `to2@domain.com`, `cc1@domain.com`, and `cc2@domain.com`.

On `main`, these values are published as four complete row-aligned recipient records and assembled into the original fixture's readable EML.

The Tika attachment fixture emits seven top-level messages assigned by exact contents-table rows to `/Début du fichier de données Outlook`, plus one separately linked embedded child, nine directly owned recipient records, ten body records, six valid body payloads totalling 271 bytes, two explicit unresolved HTML forms, two attachment records, two exact attachment payloads totalling 12,315 bytes, and two deterministic EML files. The method-`5` payload is byte-identical to the 453-byte standalone child EML and uses `message/rfc822`. The attachment owner retains its 17,035-byte `multipart/mixed` EML and byte-identical DOCX; method `5` is deliberately not inserted into the parent MIME tree.

The java-libpst comparison fixture remains a deterministic fail-closed baseline: 25 folders, 9 message metadata records, 12 body records, 0 recipients, 22 attachment metadata records, 0 materialised attachment payloads, 0 validated `IPM.Note*` classes, and 0 EML files. It is comparison evidence rather than an email capability milestone.

## Progress over time

| Phase | Outcome |
|---|---|
| M1-M25 | Built the local/Docker product foundation, command surface, structured output contract, batch operations, diagnostics, and handoff documentation. |
| PQ1-PQ35 | Corrected root and index traversal, identified real folders/messages, investigated property and subnode paths, and replaced invalid table assumptions with measured evidence. |
| PQ36 | Produced the first major fidelity improvement by decoding permitted blocks, rejecting false table declarations, recovering text/RTF bodies, and reducing unknown properties. |
| PQ37-PQ57 | Resolved the real Table Context heap, row-index BTH, subnode-backed row storage, four 52-byte rows, and exact bounded bitmap masks. |
| PQ58-PQ74 | Validated descriptor mapping, constructed bounded row transport, decoded supported fixed-width MAPI values, and integrated fail-closed diagnostics into production reporting. |
| Vertical 1-28 | Progressed from recipient property classification to structured recipients, readable body forms, and one deterministic plain/HTML EML. |
| Vertical 29-31 | Recovered `attachment.docx` metadata, its data-tree reference, and the exact validated 11,862-byte DOCX payload. |
| Vertical 32 / #452 | Bridged heap-backed row matrices into production and emitted eight Tika recipients with direct message ownership. |
| Vertical 33 / #454 | Assembled the first deterministic Tika attachment EML from validated Date, recipient, plain-text body, and DOCX evidence. |
| Vertical 34 / #455 | Parsed the exact PtypObject wrapper, resolved one unique normal-message NID, and emitted the linked child message, one recipient, and two raw body records without changing the outer EML. |
| Vertical 35 / #457 | Emitted the linked child as an exact 453-byte attachmentless `text/plain` EML, authorised from attachment metadata and excluding unrelated unvalidated plain-only messages. |
| Vertical 36 / #461 | Materialised those exact bytes as the method-`5` `message/rfc822` payload, preserved the parent EML/DOCX, and added fail-closed link and nesting tests. |
| Vertical 37 / #464 | Corrected table NID classification, decoded seven exact physical contents-table rows, assigned every top-level Tika message to its authoritative Unicode folder, and retained explicit unassigned behavior for unsupported evidence. |
| Vertical 38 / #470 | Rejected four-byte binary body locators, emitted explicit unavailable HTML records, retained valid plain-text siblings, and kept attachment and EML payloads byte-identical. |
| Vertical 39 / #473 | Decoded ANSI NBT/BBT root offsets as 32-bit fields and the ANSI crypt-method byte for diagnostics only; synthetic version-14/15 tests ensure those values cannot enable traversal. |

Detailed point-in-time milestone and experiment records are retained under `docs/`. They are historical evidence, not the current roadmap.

## What works

```text
pstd version
pstd inspect --input <pst-file>
pstd inspect --input <pst-file> --json
pstd extract --input <pst-file> --output <output-dir>
pstd batch --input <pst-file-or-directory> --output <output-dir>
python -m pstd --help
```

Implemented capabilities include bounded parsing of PST headers, BBT/NBT pages, blocks, subnodes, Heap-on-Node allocations, BTH structures, Property Contexts, Table Contexts, row storage, selected MAPI values, folder/message candidates, bodies, recipient evidence, structured outputs, batch state, and public-fixture diagnostics. ANSI header values are diagnostic-only; ANSI traversal and extraction remain backlog-only.

## Microsoft Purview corpus target

Purview is the primary producer target, but no controlled Purview export has yet been admitted. The first fixture must use a synthetic mailbox, retain immutable original PST bytes, and record its exact byte length, SHA-256, header/encryption classification, independent inventory, repeated PSTD result, completeness counts, ownership, payload hashes, MIME structure, and explicit unavailable or unsupported evidence.

The preferred first new capability is multiple by-value attachments with exact ownership. If the first admissible export instead proves inline Content-ID correlation, authoritative Exchange-to-SMTP mapping, another embedded-message layout, or broader HTML/RTF evidence, the implementation should select the smallest coherent vertical supported by those bytes.

Purview coverage must be reported by fixture and capability. PSTD must not be described as generally reliable for Purview exports until a representative controlled corpus passes without silent data loss.

## Dependency boundary

PSTD must not add java-libpst, libpst, libpff, Apache Tika, Outlook, or another PST parser/converter as a required library, build, runtime, normal test-runtime, Docker, Python-wrapper, or end-user dependency. Pinned external implementations may be used offline or in explicitly isolated fixture-generation and comparison workflows to create controlled fixtures and independently inventory counts, ownership, properties, payload bytes, hashes, and MIME structure. Any committed fixture still requires documented provenance or a reproducible generation recipe, redistribution permission, immutable bytes, byte length, and SHA-256. PSTD acceptance must come from its own Rust implementation and exact fixture output; agreement with another parser is supporting evidence, not authoritative truth.

## Important limitations

PSTD is not yet a general-purpose or absolute-coverage PST-to-EML converter. Current evidence is fixture-limited. No approved Microsoft Purview export fixture is committed. The Tika sender remains a raw native Exchange distinguished name rather than resolved SMTP; one method-`5` layout is exact, but nested child attachments, broader Unicode producers, inline attachments, real ANSI traversal, and uncommon/corrupt layouts remain incomplete. Do not infer broad compatibility from the milestone count.

## Validation gate

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo run -- --help
cargo run -- version
cargo run -- inspect --help
cargo run -- batch --help
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
```

Approved fixtures must also pass inspect, extract, batch, deterministic output, and exact artifact checks. Never commit private PST data.

## Documentation

- [Documentation index](docs/README.md)
- [Current project status](docs/product/project-status.md)
- [Compatibility matrix](docs/product/compatibility-matrix.md)
- [Microsoft Purview Unicode corpus plan](docs/operations/purview-unicode-corpus-plan.md)
- [Public PST progress log](docs/operations/public-pst-progress-log.md)
- [Extraction roadmap](docs/product/pstd-v1-roadmap.md)
- [System overview](docs/architecture/system-overview.md)
- [Codebase map](docs/engineering/codebase-map.md)
- [Developer guide](docs/engineering/developer-guide.md)
- [Local validation](docs/operations/local-validation.md)
- [Output contract](docs/data/pstd-v1-output-contract-summary.md)
- [Unsupported and deferred areas](docs/operations/v1-unsupported-deferred-areas.md)
- [Documentation status and history policy](docs/DOCUMENTATION_STATUS.md)
