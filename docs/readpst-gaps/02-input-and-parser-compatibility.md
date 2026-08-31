# Input and parser compatibility

`readpst` delegates the format work to libpst. The compatibility target therefore includes the input families and parser behaviours that are observable through successful extraction, diagnostics, or safe failure.

## File families

| Input family | libpst/readpst behaviour | PSTD status | Closure evidence |
|---|---|---|---|
| 32-bit ANSI PST, NDB version 14/15 | Reads pre-Outlook-2003 32-bit index and descriptor structures. | **Partial**: variant-correct header, 32-bit root, and BBT/NBT traversal are integrated through the production inspect/extraction loaders. | Controlled ANSI v14 evidence, malformed/strong-crypt negatives, repeat-run equality, and a broader semantic ANSI corpus for full item/output promotion. |
| 64-bit Unicode PST, NDB version 23/24 paths | Reads Outlook 2003+ Unicode PST structures. | **Partial**: bounded Unicode traversal and several real fixtures are validated. | Multiple producers, large files, mixed item classes, and exact completeness counts. |
| 64-bit OST 2013 path | libpst 0.6.71 added support for the OST 2013 format. | **Partial**: explicit OST 2013 family detection, 4 KiB roots, and 16-bit/64-bit BBT/NBT traversal are integrated through the production path. | Controlled OST 2013 evidence, truncation/malformed negatives, repeat-run equality, and a broader semantic OST corpus for full item/output promotion. |
| Files larger than 2 GiB | Uses large-file-safe offsets and 64-bit sizes where supported by the build. | **Partial**: internal offsets are wide, but large-file behaviour is not fixture-proven. | Sparse/large synthetic fixture and a real large-file performance run without overflow or whole-file loading. |
| Empty, truncated, corrupt, or malicious input | Must not produce unbounded reads or silently valid-looking content. | **Partial**: bounded reads and diagnostics exist. | Differential corruptions for every parser stage with stable error codes and no partial ownership claims. |

## RP-M1-01 delivery

PSTD now projects a bounded `InputCapability` at the parser boundary. It classifies the libpst index types for Unicode, ANSI, and OST 2013, records crypt method and root-pointer readiness, preserves the ISO-8859-1 default charset policy, and exposes file/read/candidate/property/diagnostic/depth budgets. Unsupported families and unsupported crypt methods, short headers, invalid roots, and budget violations are explicit statuses; they do not become an empty folder tree. Inspect JSON and canonical extraction archives carry the capability record.
## RP-02D charset slice

The MAPI decoder now applies a validated per-context charset from `PR_MESSAGE_CODEPAGE` and `PR_INTERNET_CPID` when supported (ISO-8859-1, Windows-1252, UTF-8, Shift-JIS/932, GBK/936, EUC-KR/949, or Big5/950), otherwise using the configured run-level `-C` fallback and retaining raw code-page evidence plus resolution provenance. Conflicting, malformed, out-of-range, and unsupported declarations fail closed to the configured fallback; an explicit `-C` override is authoritative. The selected charset reaches message and folder property contexts, table-row attachment contexts, and recursively recovered embedded messages. NUL-terminated legacy bytes, including high-bit values, remain explicit and deterministic. The pinned `encoding_rs` decoder reports malformed multibyte conversion counts without discarding raw bytes. Item-level charset properties and broader producer corpus evidence remain open.

## RP-M1-04 provenance boundary

The canonical extraction archive now emits data/evidence.jsonl. Each record keeps a
stable owner and source reference for decoded MAPI properties, subnode references,
body payloads, and attachment payloads, together with a SHA-256 digest and bounded
raw-byte retention. Property-load and unavailable-payload failures remain explicit
evidence statuses; the bounded raw field is not a claim that an oversized value was
fully retained.

## RP-M6-01 delivery

The ANSI v14/v15 and OST 2013 input boundary is now integrated in `src/pst/header.rs`,
`src/pst/layout.rs`, `src/pst/bbt.rs`, `src/pst/nbt.rs`, `src/pst/inspect.rs`, and the
canonical metadata extraction path. ANSI uses 512-byte pages, 8-bit page counts,
32-bit node/block identities, and 32-bit root offsets; OST 2013 uses 4096-byte pages,
16-bit page counts, 64-bit identities, and Unicode-width root offsets. The legacy
public loaders retain their Unicode behaviour while production callers select the
layout from the parsed family.

The permanent workflow `.github/workflows/readpst-ansi-ost.yml` generated controlled
byte-level fixtures and passed run `32508581805` plus the full CI run `32508581926` at
branch head `86c91304a859aae1d31f5974f7873d2a6ecbb514`. Fixture hashes are recorded in
the uploaded artifact (`readpst-ansi-ost-evidence`, artifact `9456174707`, ZIP digest
`cbb0bc2408de09a0294ac864352d066d7a421f5941d48e90f02774226f0cea04`): ANSI v14
`faa9e0f8c5fcee7abbdf4078d25d993ecfcba16889d6739deb7787e9ade1dfe`, OST 2013
`d600489f97a0e4123a4d9b389e72c0bc1edd48e5b908fda496d23b2f9f42262d`, strong-crypt
ANSI `55b2a5ec86c3b4df896e8aebd8f1098b02d860244ae398f5df134740832d8ac9`, truncated
OST `2849147f44eae7169efa85cf027aeb04ed1eb69513ac50c32d37ca144f33c7d3`, and
malformed-short `44a1460f6df89ff23b880552ed878044b97d26074ea1d63d3b79a18091d81e4d`.
The positive fixtures prove canonical family/root/index identity and repeated inspect
JSON equality. Strong crypt is now `ready` at the capability boundary; the production
payload loader decodes the pinned method-2 transform. Unknown methods remain explicit
`unsupported`, truncation is explicit `partial`, and a short malformed file exits
non-zero. This is still structural input breadth, not a claim that every ANSI/OST
message, property, or output mode is complete.

## Encryption

The libpst header exposes three states:

| Mode | Upstream behaviour | PSTD status |
|---|---|---|
| no encryption | Read bytes normally. | **Implemented** for validated Unicode evidence. |
| compressible encryption | Apply libpst’s fixed substitution table before interpreting data. | **Partial**: bounded permute decoding is implemented for validated Unicode payload paths. |
| “strong” encryption | Apply the fixed three-rotor transformation used by libpst. | **Partial**: production payload decoding and a pinned known vector are covered; broad encrypted item/output corpus remains. |

PSTD applies the bounded permute table for crypt method 1 and the block-ID salted
three-table transform for method 2 in the canonical payload reader. Method 2 is not a
password scheme in the pinned libpst branch, so PSTD does not invent a password result;
corrupt or semantically undecodable encrypted payloads retain bounded raw evidence and
fail explicitly downstream. Unknown methods remain unsupported. A file must not be
labelled successfully extracted merely because the header can be classified.

The hardening boundary also rejects symlink input paths, does not traverse symlinked
directories during batch discovery, caps recursive discovery depth and admitted PST
file count, enforces checked reader offsets and single-read limits, and caps emitted
diagnostic records. These are explicit PSTD safety improvements over libpst's
allocation/exit-oriented helper behavior and do not change the default crypt semantics.

## Index, node, and property behaviours

Parity requires the Rust parser to cover the structures that readpst uses to obtain an item:

- both 32-bit and 64-bit NBT/BBT entry shapes;
- root and child B-tree pages, offsets, sizes, and back pointers;
- normal, associated, contents, hierarchy, search, and attachment-related NIDs;
- descriptor-tree parent/child relationships and folder ownership;
- Heap-on-Node allocations, BTH indexes, Property Contexts, Table Contexts, and subnode trees;
- extended attribute/name mappings for named MAPI properties;
- direct, subnode-backed, XBLOCK, and multi-level data payloads;
- attachment ID2/reference resolution and embedded-object linkage;
- the libpst “no object”/missing reference cases without crashing.

PSTD has substantial Unicode parser groundwork in these areas, including bounded table-row transport, selected fixed-width MAPI decoding, subnode traversal, and one validated XBLOCK attachment path. The gap is breadth: the current selected-property approach is not yet a general libpst-equivalent item decoder.

## Character-set interpretation

libpst tracks whether each string is Unicode and chooses a fallback from item charset, message code page, internet CPID, or the file default. It converts strings to UTF-8 for output and handles code pages through iconv. PSTD must add:

- explicit String8/StringUnicode handling across every selected and typed field;
- code page and internet-CPID mapping;
- per-property charset provenance;
- a run-level fallback charset equivalent to `-C`;
- conversion failure and replacement policy that never changes a raw value into a falsely authoritative value;
- UTF-7 and other folder/property encodings encountered in real PSTs.

## Concurrency and resource safety

readpst can fork work across folders or individual messages in separate mode and reopens the PST in child processes. PSTD should prefer bounded worker concurrency that is safe for a Rust reader, but parity requires equivalent throughput controls and deterministic aggregation rather than a particular process model.

Every input-parity implementation must report file size, header family, crypt method, selected roots, index/traversal counts, parser limits, and scoped failures in the existing inspect/summary contract.

## Planned implementation — `RP-02`

### Readpst/libpst logic reviewed

`pst_open` validates the file magic, selects 32-bit, Unicode 64-bit, or the 4 KiB/OST 2013 layout, reads the encryption method and root pointers, and records the file size and charset. `pst_load_index` builds the ID and descriptor indexes; `pst_load_extended_attributes` reads named-property metadata; `pst_getTopOfFolders` resolves the personal-folder root or the OST fallback. `pst_reopen` is used by readpst’s forked children. `pst_parse_item` loads the ID2 tree, processes the property context, recognizes DSN/MDN and attachment tables, resolves attachment data in a second pass, and deep-copies the child ID2 context. `pst_attach_to_file` and its base64 variant read either in-memory or ID-backed data. The conversion path uses `pst_default_charset`, `pst_convert_utf8*`, and iconv; `lzfu.c` handles compressed RTF separately.

The current PSTD path already has `PstByteReader`, `PstHeader`, `BbtIndex`, `NbtIndex`, `LogicalNodeStore`, `data_tree`, `subnodes`, `PropertyContext`, `TableContext`, parser limits, and explicit crypt diagnostics. `RP-02` fills the breadth and turns diagnostics into validated input capabilities rather than copying libpst internals.

### Planned PSTD modules and records

Extend the parser boundary with:

```text
src/pst/format.rs       PstFamily { Ansi32, Unicode64, Ost2013 }
src/pst/crypto.rs       CryptMethod and bounded byte transforms
src/pst/charset.rs      code page/CPID/fallback resolution and provenance
src/pst/input.rs        validated open/index/root contract
src/pst/limits.rs       decompression, recursion, payload, and page budgets
src/pst/evidence.rs     source offsets, raw bytes, and rejection reasons
```

`PstHeaderSummary` should expose family, crypt method, code page/charset evidence, root pointers, and file size. A parser error must carry stage, node/block/property identity, byte range, and a stable reason code. The reader must never return a successful typed item when a required index or ownership edge is only guessed.

### Implementation flow

1. Parse the header using an explicit family decoder. Validate all size/offset arithmetic against file length and `ParserLimits` before reading.
2. Select the BBT/NBT entry width and page format; load parent pages recursively with visited-page sets and count every rejected or unreachable page.
3. Decode the message-store and folder roots, including the OST 2013 fallback only when the family and node evidence justify it.
4. Apply the crypt method at the byte-reader layer. Implement compressible and strong transformations as isolated, table-driven modules with known-vector tests; do not let output code know about encryption.
5. Resolve descriptor children, heaps, BTH/property/table contexts, subnodes, XBLOCK/XXBLOCK chains, and ID2 references through bounded plans. Preserve raw blocks when a typed interpretation fails.
6. Load extended/named attributes before projecting property names. Track `(property tag, value type, source encoding, raw bytes, conversion status)` per value.
7. Resolve charset in readpst’s order—body/item charset, message code page, internet CPID, file charset, ISO-8859-1 fallback—but expose the selected source and allow `RP-01`’s explicit run override.
8. Emit a parser report containing counts and scoped failures. Only then hand nodes to the folder/item envelope.

### Improvements over readpst

- Keep transformations pure and per-reader instead of relying on mutable library/global conversion state.
- Check compressed RTF declared size, input bounds, and output budget before allocation. The upstream LZFU routine trusts the declared raw size and has no CRC validation.
- Make crypt, family, and charset decisions explicit in evidence; do not infer success from a readable header alone.
- Preserve raw String8/Unicode bytes and conversion failures, including embedded NUL and malformed UTF-16 cases, rather than only retaining converted strings.
- Use checked arithmetic and visited sets for all page, node, subnode, and block chains.
- Keep OST and ANSI support behind capability gates with fixtures, so partial support cannot masquerade as general PST support.

### Issue-ready acceptance

Split the work into `RP-02A` (family/header/index), `RP-02B` (crypt), `RP-02C` (properties/subnodes/references), `RP-02D` (charset), and `RP-02E` (limits/fuzz hardening). Each issue must include:

- a qualifying positive fixture and SHA-256/provenance record;
- an equivalent libpst/readpst invocation or source-level expected result;
- corrupt/truncated/overflow derivatives for the touched stage;
- inspect/summary assertions for family, crypt, roots, counts, and reason codes;
- repeat-run equality and bounded-memory/size assertions;
- updates to [folders and item types](03-folders-and-item-types.md), [metadata](04-message-metadata-and-headers.md), [bodies](05-body-mime-and-rtf.md), [attachments](06-attachments.md), and the matrix.
