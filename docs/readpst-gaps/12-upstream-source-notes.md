# Upstream source notes

This page makes the register auditable. Links are pinned to the libpst commit used for the comparison.

## `readpst.c`

| Area | Source location | Observation used by this register |
|---|---|---|
| CLI parsing and options | [`main`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/readpst.c#L419-L584) | Flags for charset, deleted items, debug, parallelism, output modes, output types, overwrite, UTF-8 preference, and output directory. |
| Root opening and folder traversal | [`main` and `process`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/readpst.c#L243-L405) | Message store, top-of-folder lookup, recursive children, mixed item types, deleted-folder policy, and typed routing. |
| Output layout creation | [`create_enter_dir`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/readpst.c#L2355-L2448) | Normal, KMail, recursive, separate, Thunderbird, file naming, uniqueness, and per-folder streams. |
| Output layout closing/counts | [`close_enter_dir`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/readpst.c#L2450-L2490) | Empty-file removal, item/skipped/stored counts, and Thunderbird sidecars. |
| Attachment filtering and files | [`acceptable_ext` and `write_separate_attachment`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/readpst.c#L1031-L1112) | Extension allow-list, long/short names, generated names, collisions, and separate payload files. |
| Embedded messages | [`write_embedded_message`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/readpst.c#L1115-L1157) | ID2 resolution, child parsing, `message/rfc822`, and safe skip of non-email children. |
| MIME attachment parts | [`write_inline_attachment`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/readpst.c#L1251-L1308) | Default MIME type, base64, Content-ID, long/short filename handling, RFC 2231, disposition. |
| Header validation and normalization | [`valid_headers` helpers](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/readpst.c#L1310-L1577) | Folded header detection, invalid-body rejection, field extraction, charset/report-type extraction, and stripped duplicates. |
| MIME body writer | [`write_body_part`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/readpst.c#L1579-L1642) | Charset conversion, text transfer encoding, base64 detection, and body-part boundaries. |
| Ordinary email writer | [`write_normal_email`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/readpst.c#L1675-L2004) | Header reconstruction, read status, forensic headers, mixed/alternative/report MIME, RTF/encrypted-body synthetic attachments, schedule parts, and attachment dispatch. |
| Contact output | [`write_vcard`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/readpst.c#L2005-L2170) | vCard field projection, escaping, notes, categories, phones, addresses, and version. |
| Journal and appointment output | [`write_journal` and `write_appointment`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/readpst.c#L2170-L2354) | vJournal/iCalendar fields, recurrence, labels, categories, alarms, and timestamps. |

Line ranges are pointers into the pinned revision; the named functions are the stable reference if upstream formatting changes.

## `libpst.h`

The header defines the requirements that are otherwise easy to miss in `readpst.c`:

- item types and encryption modes near the top of the file;
- attachment method constants and message flags;
- the full email model, including report, RTF, encrypted body, representing-party, and forensic fields;
- contact, journal, recurrence, appointment, and attachment structures;
- file metadata for 32/64-bit families, OST 2013, encryption, indexes, and charset defaults.

See the pinned [header](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/libpst.h) rather than copying only the fields already used by PSTD.

## `libpst.c`

The library source establishes implementation-level requirements for:

- opening and classifying PST/OST files;
- reading encryption state and applying the two supported transformations;
- loading both index trees and extended attributes;
- resolving attachment references and payload chains;
- converting per-property strings to UTF-8;
- deriving item types from message/container classes;
- decoding recurrence data.

The relevant source is [`src/libpst.c`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/libpst.c).

## Regression categories

The upstream regression list includes samples for:

- ANSI and Unicode-era files;
- HTML, plain text, RTF, and embedded images;
- non-ASCII subjects and character sets;
- appointments and recurring appointments;
- MIME-signed email;
- embedded RFC 822 messages;
- journal archives;
- large and unusual archives;
- contacts/distribution-list related data.

PSTD should reuse these categories as fixture requirements without importing private or impermissibly licensed payloads.

## Complete source-review ledger

The following ledger records the direct source files and scripts reviewed for the readpst capability surface. “Applied in PSTD” is the planned Rust boundary, not a claim that the code already exists.

| Upstream file | Reviewed symbols/regions | Behaviour captured | Applied in PSTD |
|---|---|---|---|
| `src/readpst.c` | `main`, `process`, `try_fork`, `create_enter_dir`, `close_enter_dir`, `mk_*`, header helpers, body/MIME writers, contact/journal/appointment writers | complete CLI, traversal, routing, output modes, MIME, attachments, typed non-mail output, counts | `RP-01`–`RP-09`; `ItemEnvelope`, profiles, MIME tree, typed serializers |
| `src/libpst.c` | `pst_open`, `pst_reopen`, `pst_load_index`, `pst_load_extended_attributes`, `pst_getTopOfFolders`, `pst_parse_item`, `pst_process`, attachment helpers, charset and recurrence helpers | input family, encryption, indexes, properties, item projection, ID2/reference resolution, conversions, recurrence | `RP-02`–`RP-08`; parser evidence and typed projections |
| `src/libpst.h` | type/method/flag constants and email/contact/appointment/journal/store/attachment structures | the full semantic model that `readpst.c` consumes | `RP-03`–`RP-08`; Rust enums/records with raw-property retention |
| `src/msg.cpp`/`src/msg.h` | property helpers; `write_msg_email` 203–434; recipient/attachment loops; NameID stream | OLE MSG compound-document layout, MAPI property type/stream mapping, known embedded-message omission | `RP-09E`; dedicated OLE writer and property matrix |
| `src/lzfu.c`/`src/lzfu.h` | `pst_lzfu_decompress` 38–120; Outlook dictionary/header/flag loop | compressed RTF expansion, declared-size trust, missing CRC/budget checks | `RP-05D`; bounded Rust decompressor with raw/synthetic status |
| `src/vbuf.c`/`src/vbuf.h` | buffer growth; `pst_unicode_init/close`; UTF-16↔UTF-8 and UTF-8↔target charset | iconv conversion lifecycle, String8/Unicode conversion, failure behaviour | `RP-02D`, `RP-04E`; per-run Rust charset resolver |
| `src/timeconv.c`/`src/timeconv.h` | FILETIME-to-Unix/ASCII/string/UTC `tm` functions | timestamp conversion, local/UTC formatting, 32-bit overflow fallback | `RP-04E`, `RP-08B`; typed FILETIME provenance and explicit fallback |
| `src/libstrfunc.c`/`src/libstrfunc.h` | `pst_base64_encode*`, 76-column line handling | body/attachment base64 formatting and edge cases | `RP-05C`; standards-tested streaming transfer encoder |
| `src/debug.c`/`src/define.h`/`src/common.h` | debug macros, allocation/error helpers, portability definitions | global diagnostics, memory/error conventions, platform assumptions | `RP-00`, `RP-01`; structured bounded diagnostics and checked Rust errors |
| `src/XGetopt.c`/`src/XGetopt.h` | conditional fallback `getopt` implementation wired by `Makefile.am`/`define.h` | Windows/portable CLI option parsing when libc `getopt` is unavailable | `RP-01`; use Rust’s parser with equivalent option grammar and platform-independent errors |
| `src/Makefile.am` | `common_source`, `readpst_SOURCES`, `readpst_LDADD` 29–95 | direct build dependency boundary: readpst plus libpst/common helpers and GSF | `RP-12`; dependency/license review, no libpst linkage in PSTD |
| `regression/regression-tests.bash` | `consistency`, `dodii`, `doldif`, `dopst`, profile commands 51–100, fixture list 126–156 | operational profiles, valgrind/size gates, output cleanup, fixture categories, semantic regression mode | `RP-11`, `RP-13`; fixture manifest and differential runner |
| `NEWS`/`ChangeLog` | release notes for encryption type 2, OST 2013, Content-ID, RFC 2047/2231, reports, embedded messages, recurrence, mixed types | historical fixes that may not be obvious from current call sites | `RP-12`; drift review and regression requirements |

### `msg.cpp` application notes

The `.msg` writer is reviewed separately because it is easy to overstate parity from the `-m` flag. Its property record is a five-field fixed layout (`type`, `id`, `flags`, `length/value`, `reserved`). String properties create `__substg1.0_<id><type>` streams and append terminators for ANSI/Unicode types; file-backed properties stream bytes in 10,000-byte chunks. `write_properties` emits `__properties_version1.0`. The top-level writer emits selected importance, priority, sensitivity, receipt, flags, sent date, subject, sender/recipient, transport header, body, HTML, message ID, and reply-to properties. It creates recipient storages with To/Cc/Bcc recipient types and attachment storages with by-value data, names, MIME type, rendering position, MIME sequence, and record key. It creates three empty NameID properties but does not populate arbitrary named properties or embedded attachments.

PSTD should preserve this observable compatibility map while fixing the upstream writer’s hard-coded `iso-8859-1//TRANSLIT//IGNORE`, destructive string conversion, temporary filename, limited recipient projection, and silent embedded-attachment omission. The issue must state which MSG properties are supported, raw-preserved, or unsupported.

RP-M5-04 implements the supported boundary in `src/output/msg.rs` without linking or
porting GPL code: a Rust-native CFB/OLE writer emits UTF-16 string streams, FILETIME,
numeric/boolean flags, root properties, recipient storages, by-value attachment
storages, and the three empty NameID streams observed upstream. It preserves canonical
raw evidence while marking invalid scalar/date values, missing payloads, method-5
embedded attachments, and method-6 materialization decisions. The independent workflow
uses `olefile` to open every output and compare subject, recipient roles, attachment
payload hashes, path safety, and repeated bytes.

### Regression-script application notes

`regression-tests.bash` is not a parser specification, but it is an operational source of truth. `dopst` demonstrates recursive/contact mode, `-8`, `-a`, `-C`, `-j`, `-r`, `-m`, `-S`, `-D`, and logging combinations; the fixture list names HTML/text, ANSI-era, large, appointment, MIME-signed, embedded RFC 822, non-ASCII, RTF, and journal cases. `consistency` compares property constants to XML documentation. The script also removes output files in regression mode and filters a known unstable token before comparison.

PSTD’s replacement must make these implicit assumptions explicit: fixture manifests replace positional filenames; cleanup is scoped to a temporary run root; semantic comparison replaces grep/file deletion; valgrind becomes Rust sanitizer/resource-budget coverage; and `jobs=0` becomes a documented bounded worker policy. Utility functions `dodii` and `doldif` are recorded for source-boundary awareness but are not readpst parity requirements.

## RP-M0-03 implementation

The source-review contract is implemented in `tests/readpst_diff/source_manifest.rs` and exported through `tests/readpst_diff/mod.rs`. It records all 22 direct source/script/release-note paths in this ledger, their selected symbols or stable behaviour anchors, the 28 work-unit mappings, and the eight regression profile categories. Every generated source URL is pinned to `cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89`; a revision mismatch, missing/duplicate path, unresolved work-unit symbol, or changed repository/license boundary produces an actionable failure. The check is deterministic and has no network or private-fixture dependency. Sibling utilities (`lspst`, `pst2ldif`, `nick2ldif`, and `pst2dii`) remain explicitly out of scope for readpst parity.

## RP-M7 source review checkpoint

The release review revalidated this ledger at pinned revision
`cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89` before reading the live matrix on main
`57fbcaf1a83e2ddc79fff300be812a23cc66bb53`. The remaining 19 Gap rows are all
readpst-exposed behaviours from these anchors, not upstream skips. RP-M7-02 must
record unavailable or inadmissible regression profiles explicitly; it must not use a
moving upstream checkout or silently convert missing corpus evidence into parity.

RP-M7-03 final decision: the pinned source review confirms that the remaining 19 Gap
rows are readpst-exposed behaviours, while the 54 Partial rows lack breadth or E4
evidence. The source ledger therefore supports a NOT PARITY-COMPLETE release decision,
not an upstream-skip classification.

## Planned implementation — `RP-12`

1. Keep this ledger pinned to `cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89` and add a new review row whenever upstream changes the readpst dependency boundary.
2. For each `RP-*` issue, copy the exact source anchor and ledger row into the issue body; cite observable behaviour, not GPL implementation text.
3. Add source-drift checks that compare the pinned revision, exported function names, option list, item/method constants, helper inventory, and regression fixture categories.
4. When a PSTD implementation intentionally improves readpst, document both the compatibility projection and the stronger canonical behaviour in the topic page, matrix, and differential comparator.
5. Re-review tangential documents after any change to this ledger. A changed helper such as charset conversion, LZFU, attachment references, or MSG property mapping can invalidate multiple plan pages.

## RP-M6-01 source application note

The pinned `pst_open` family branches and index-loading behaviour are applied without
copying libpst code. PSTD now selects a variant-specific layout before production BBT/NBT
loads: ANSI v14/v15 uses the 32-bit root and entry widths, while OST 2013 uses the 4 KiB
page and Unicode-width root/index identities. The source baseline still requires later
semantic item/property/output corpus work; structural page admission is not promoted as
full readpst extraction parity. The controlled fixture workflow records the positive and
negative evidence for this boundary.

`RP-12` is complete for a release only when every upstream source file that is linked into readpst is either mapped to a PSTD module/fixture or explicitly declared out of scope with a reason. The current direct dependency set is the set listed above; sibling utilities are not silently treated as readpst requirements.

### Pinned helper links

These links make the lower-level review directly actionable from an issue:

| Behaviour | Pinned source |
|---|---|
| Open/reopen, indexes, and extended attributes | [`pst_open`/`pst_reopen`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/libpst.c#L315-L409), [`pst_load_index`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/libpst.c#L657-L748), [`pst_load_extended_attributes`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/libpst.c#L703-L748) |
| Root, item parse, attachment passes, and MAPI projection | [`pst_getTopOfFolders`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/libpst.c#L549-L603), [`pst_parse_item`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/libpst.c#L1254-L1410), [`pst_process`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/libpst.c#L2186-L2440) |
| Attachment byte and base64 extraction | [`pst_attach_to_file*`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/libpst.c#L605-L655) |
| Charset and RFC encodings | [`pst_default_charset`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/libpst.c#L4458-L4470), [`pst_rfc2231`/`pst_rfc2047`/UTF-8 conversions`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/libpst.c#L4471-L4565) |
| Compressed RTF | [`pst_lzfu_decompress`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/lzfu.c#L38-L120), [`lzfu.h`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/lzfu.h) |
| Charset buffers | [`vbuf.c`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/vbuf.c#L18-L258), [`vbuf.h`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/vbuf.h) |
| FILETIME conversion | [`timeconv.c`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/timeconv.c#L1-L34), [`timeconv.h`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/timeconv.h) |
| Base64 line wrapping | [`libstrfunc.c`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/libstrfunc.c#L1-L73), [`libstrfunc.h`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/libstrfunc.h) |
| MSG OLE writer | [`msg.cpp` helpers and `write_msg_email`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/msg.cpp#L20-L434), [`msg.h`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/msg.h) |
| Regression operations and fixture categories | [`regression-tests.bash`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/regression/regression-tests.bash#L1-L167), [`src/Makefile.am`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/Makefile.am#L20-L95) |
