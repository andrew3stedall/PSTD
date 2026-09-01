# PSTD Project Status

_Last reviewed: 1 September 2026._

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
| Embedded message | Partial bounded recovery | One separately linked child remains exact for the approved layout; child attachment subnodes and nested method-5 children are now walked under the depth budget, while additional producer layouts remain unproven. |
| Folder/message ownership | Exact on the Tika fixture | Eight folders and seven top-level physical message owners resolved from authoritative contents-table rows; the embedded child remains isolated. |
| Independent body forms | Exact on approved fixtures | Four-byte Property Context body locators remain explicit unavailable forms; valid plain-text siblings are retained independently. |
| ANSI input baseline | Stage B message plus Stage C by-value attachment exact for two controlled shapes | Deterministic Linux Rust v14 fixtures cover one `Synthetic Mail` folder/message with a plain-text body and structured To row, plus one ANSI/String8 property-context method-1 attachment with exact filename, MIME, payload hash, canonical bytes, inline MIME/base64, external raw-file/manifest, and independent validation. Broad ANSI producers, reference/embedded/OLE attachments, HTML/RTF, and typed items remain unproven. |
| Microsoft Purview Unicode exports | Active corpus target | No approved Purview export fixture is yet committed. Compatibility must be established capability-by-capability on controlled synthetic Purview exports rather than inferred from the existing fixtures. |
| External PST implementations | Comparison-only tooling | Pinned external tools may generate or independently inventory controlled fixtures, but PSTD acceptance must come from its own Rust implementation and exact deterministic output. |
| Downstream systems | Parked | Snowflake, UI, search, analytics, semantic search, and graph work remain out of scope. |
| Readpst parity release gate | Attachment payload extraction wave | RP-M7-03 remains NOT PARITY-COMPLETE for its reviewed baseline; the maintained matrix is now 10 Implemented, 54 Partial, and 11 Gap after generic direct/data-tree extraction, method-aware metadata, MIME sequence ordering, and bounded nested-child recovery. |
| Pinned semantic differential | RP-M7-02 evidence collected | Run `32512518536` passed the 18-test readpst/PSTD harness for the approved Unicode fixture; release-wide E4 remains not proven because admissible profile/input corpus coverage is incomplete. |

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
| EML files / bytes | 2 / deterministic; inline parent carries both recovered payloads |
| Messages JSONL bytes | 23,865 |
| Bodies JSONL bytes | 2,922 |
| Recipients JSONL bytes | 2,708 |
| Attachments JSONL bytes | 1,240 |
| Extraction TAR bytes | 234,496 |

The method-5 record `att_a9c94a13d70f1cb3` publishes a 453-byte `message/rfc822` payload with SHA-256 `86ffe5567da7aa505b8be16400889170ca583fd247cc0758f00a43c2a8a99420`. Those bytes are identical to standalone child `msg_0ff529af59d373d5.eml`. Inline parent EML now carries both the validated DOCX and recovered embedded-message payload; external mode publishes the same bytes at manifest-linked paths.

## Latest completed work

Attachment reference resolution now accepts the validated compact 4-byte SLENTRY form as well as the existing Unicode wide-entry form, with ambiguity and truncation remaining explicit failures. Method-6 `PR_ATTACH_DATA_OBJ` property-context references now have exact-byte wide/compact OLE data-tree regression coverage and reject duplicate mappings; broad reference/OLE producer coverage remains Partial.

Validated property-context attachments now survive missing or blank filename properties, retaining metadata and using deterministic fallback archive names while keeping method/size validation strict. Broader attachment methods and producer coverage remain Partial.

The MAPI String8 conversion boundary now preserves legacy high-bit bytes through the documented ISO-8859-1 fallback and selects supported per-context `PR_MESSAGE_CODEPAGE`/`PR_INTERNET_CPID` declarations for UTF-8, Windows-1252, ISO-8859-1, Shift-JIS, GBK, EUC-KR, or Big5. Raw declarations and charset provenance are retained; malformed, unsupported, and conflicting metadata fails closed to the configured fallback, and `-C` remains authoritative. Malformed multibyte sequences remain raw-backed and publish explicit conversion-error counts. Broader producer-specific charset parity remains open.

The CLI fallback charset override is now effective across message, folder, attachment table/property-context, and nested embedded-message decoding. Supported names include `iso-8859-1`, `windows-1252`/`cp1252`, `utf-8`, Shift-JIS/`cp932`, GBK/`cp936`, EUC-KR/`cp949`, and Big5/`cp950`; unsupported values fail closed during policy validation.

The ANSI Stage-A structural fixture remains admitted from `tools/ansi_fixture.rs`: exact 2,048-byte length, SHA-256 `b5de1ce4cebacc2ea4cefddb4ab9c4d32e5fed04b81cd681e8831faf1323c765`, independent weak-CRC/page-trailer validation, repeat-run equality, PSTD fail-closed empty traversal, and libpff acceptance. Stage B adds a separate deterministic one-folder/one-message ANSI fixture with exact node/property/table validation, one structured recipient, a plain-text body, and EML evidence. Stage C adds one direct property-context by-value attachment with exact bytes/hash plus inline and external output evidence. These are controlled shape claims only; no broad ANSI compatibility claim is made.

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

ANSI Stage A is complete as a structural baseline. Stage B proves one controlled message extraction shape and Stage C proves one controlled by-value attachment shape; both remain below representative Purview Unicode coverage. Additional ANSI work must use separate fixtures for reference/embedded/OLE attachments, HTML/RTF bodies, non-mail items, malformed derivatives, and producer-specific layouts.

## Validation expectations

Every extraction PR must pass formatting, clippy with warnings denied, all Rust tests, CLI checks, Python wrapper checks, Docker build, approved fixture workflows, and exact artifact review. Unsupported or ambiguous candidates must remain unavailable rather than producing partial or guessed records.

Comparison workflows must identify the external implementation and pinned version used, retain raw evidence where licensing permits, and separately verify PSTD's own deterministic output. Agreement with another parser is supporting evidence, not sufficient proof when the format specification or fixture bytes contradict it.

## Risk statement

The current result is material evidence for two approved Unicode fixture paths, not broad Microsoft Purview or general PST compatibility. Purview exports may contain producer-specific folder layouts, associated contents, Exchange identities, attachment combinations, embedded messages, non-mail objects, and large-file characteristics not represented by the current fixtures. Capability claims must remain fixture-specific until a representative controlled Purview corpus passes without silent data loss.
