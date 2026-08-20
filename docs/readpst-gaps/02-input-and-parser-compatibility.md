# Input and parser compatibility

`readpst` delegates the format work to libpst. The compatibility target therefore includes the input families and parser behaviours that are observable through successful extraction, diagnostics, or safe failure.

## File families

| Input family | libpst/readpst behaviour | PSTD status | Closure evidence |
|---|---|---|---|
| 32-bit ANSI PST, NDB version 14/15 | Reads pre-Outlook-2003 32-bit index and descriptor structures. | **Gap**: header values are diagnostic-only. | A redistributable ANSI fixture plus traversal, folder, message, body, attachment, and malformed derivatives. |
| 64-bit Unicode PST, NDB version 23/24 paths | Reads Outlook 2003+ Unicode PST structures. | **Partial**: bounded Unicode traversal and several real fixtures are validated. | Multiple producers, large files, mixed item classes, and exact completeness counts. |
| 64-bit OST 2013 path | libpst 0.6.71 added support for the OST 2013 format. | **Gap** | A qualifying OST fixture and a documented decision about whether PSTD’s input contract names OST explicitly. |
| Files larger than 2 GiB | Uses large-file-safe offsets and 64-bit sizes where supported by the build. | **Partial**: internal offsets are wide, but large-file behaviour is not fixture-proven. | Sparse/large synthetic fixture and a real large-file performance run without overflow or whole-file loading. |
| Empty, truncated, corrupt, or malicious input | Must not produce unbounded reads or silently valid-looking content. | **Partial**: bounded reads and diagnostics exist. | Differential corruptions for every parser stage with stable error codes and no partial ownership claims. |

## Encryption

The libpst header exposes three states:

| Mode | Upstream behaviour | PSTD status |
|---|---|---|
| no encryption | Read bytes normally. | **Implemented** for validated Unicode evidence. |
| compressible encryption | Apply libpst’s fixed substitution table before interpreting data. | **Gap** |
| “strong” encryption | Apply the fixed three-rotor transformation used by libpst. | **Gap** |

PSTD currently reads the crypt-method field for diagnostics in parts of the parser but does not have a proven end-to-end decryption path. A file must not be labelled successfully extracted merely because the header can be classified.

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
