# PSTD Project Status

_Last reviewed: 23 July 2026._

## Purpose

Provide the authoritative view of the merged extraction baseline and the next evidence-led boundary.

## Current implementation state

| Area | Current state | Evidence and limitations |
|---|---|---|
| Product foundation | Complete through M25 | Rust CLI, Python wrapper, Docker packaging, structured TAR/JSONL output, batch/resume, diagnostics, and operator guidance. |
| Bounded PST parser | Validated foundation through PQ74 | Header, BBT/NBT, blocks, subnodes, Heap-on-Node, BTH, Property Context, Table Context, row transport, and supported MAPI values with explicit limits. |
| Original public fixture | Material readable-email path | One message, four structured recipients, text and recovered HTML, and one deterministic 956-byte EML. |
| Tika DOCX attachment | Exact | One 11,862-byte DOCX payload with validated ownership, length, hash, ZIP/CRC evidence, and deterministic parent EML placement. |
| Tika recipients | Exact or explicit native preservation | Nine directly owned recipients across the fixture, including SMTP rows and preserved legacy Exchange evidence. |
| Embedded message | Exact for one method-5 layout | One separately linked child, exact 453-byte standalone EML, and byte-identical `message/rfc822` attachment payload. Nested recursion and additional producer layouts remain unproven. |
| Folder/message ownership | Exact on the Tika fixture | Eight folders and seven top-level physical message owners resolved from authoritative contents-table rows; the embedded child remains isolated. |
| Independent body forms | Exact on approved fixtures | Four-byte Property Context body locators remain explicit unavailable forms; valid plain-text siblings are retained independently. |
| ANSI header diagnostics | Diagnostic only | Version-14/15 field offsets are decoded with variant-correct widths. ANSI traversal and email extraction remain unsupported and are not the active product priority. |
| Microsoft Purview Unicode exports | Active corpus target | No approved Purview export fixture is yet committed. Compatibility must be established capability-by-capability on controlled synthetic Purview exports rather than inferred from the existing fixtures. |
| External PST implementations | Comparison-only tooling | Pinned external tools may generate or independently inventory controlled fixtures, but PSTD acceptance must come from its own Rust implementation and exact deterministic output. |
| Downstream systems | Parked | Snowflake, UI, search, analytics, semantic search, and graph work remain out of scope. |

## Exact Tika baseline

| Metric | Current result |
|---|---:|
| Folders | 8 exact |
| Messages | 8, including one embedded child |
| Body records | 10 |
| Valid body payload files / bytes | 6 / 271 |
| Explicit unresolved HTML forms | 2 |
| Recipient records | 9 |
| Attachment records | 2 |
| Attachment payload files / bytes | 2 / 12,315 |
| EML files / bytes | 2 / 17,488 |
| Messages JSONL bytes | 23,865 |
| Bodies JSONL bytes | 2,922 |
| Recipients JSONL bytes | 2,708 |
| Attachments JSONL bytes | 1,240 |
| Extraction TAR bytes | 234,496 |

The method-5 record `att_a9c94a13d70f1cb3` publishes a 453-byte `message/rfc822` payload with SHA-256 `86ffe5567da7aa505b8be16400889170ca583fd247cc0758f00a43c2a8a99420`. Those bytes are identical to standalone child `msg_0ff529af59d373d5.eml`. The parent EML remains exactly 17,035 bytes and includes only the validated method-1 DOCX payload.

## Latest completed work

The java-libpst comparison fixture has a deterministic fail-closed baseline: 25 folders, 9 message metadata records, 12 body records, 0 recipients, 22 attachment metadata records, 0 materialised attachment payloads, 0 validated `IPM.Note*` classes, and 0 EML files. It is comparison evidence, not an email capability milestone.

## Next evidence-based milestone

Admit the first controlled, redistributable Microsoft Purview Unicode PST export and lock its exact baseline before changing parser behaviour. The source mailbox must be synthetic and the export must have documented procedure, immutable bytes, length, SHA-256, header classification, independent inventory, repeated PSTD output, and exact completeness statuses.

The first fixture should expose the smallest capability not already proven by current fixtures, preferably:

1. multiple by-value attachments with exact ownership;
2. inline attachment and verified HTML `cid:` correlation;
3. authoritative Exchange-to-SMTP mapping;
4. another embedded-message layout or bounded recursion;
5. broader independent HTML/RTF body evidence.

The complete admission and fixture-family plan is in `docs/operations/purview-unicode-corpus-plan.md`.

ANSI Stage A remains a valid later research lane, but an empty ANSI container would add no observable email or EML behaviour and therefore does not outrank representative Purview Unicode coverage.

## Validation expectations

Every extraction PR must pass formatting, clippy with warnings denied, all Rust tests, CLI checks, Python wrapper checks, Docker build, approved fixture workflows, and exact artifact review. Unsupported or ambiguous candidates must remain unavailable rather than producing partial or guessed records.

Comparison workflows must identify the external implementation and pinned version used, retain raw evidence where licensing permits, and separately verify PSTD's own deterministic output. Agreement with another parser is supporting evidence, not sufficient proof when the format specification or fixture bytes contradict it.

## Risk statement

The current result is material evidence for two approved Unicode fixture paths, not broad Microsoft Purview or general PST compatibility. Purview exports may contain producer-specific folder layouts, associated contents, Exchange identities, attachment combinations, embedded messages, non-mail objects, and large-file characteristics not represented by the current fixtures. Capability claims must remain fixture-specific until a representative controlled Purview corpus passes without silent data loss.