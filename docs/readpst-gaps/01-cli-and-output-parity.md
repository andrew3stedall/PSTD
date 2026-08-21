# CLI and output-mode parity

`readpst` exposes a single command with a compact flag set. PSTD exposes `inspect`, `extract`, `batch`, and `version`, with structured TAR/JSONL output plus named adapter profiles. The mailbox, attachment/KMail, Thunderbird, and MSG/OLE slices are now integrated; input breadth and release promotion remain explicitly partial until their dedicated work units land.

RP-M6-03 hardens the operational boundary: batch jobs are bounded to the validated
`1..=64` policy and parallel results are reassembled in sorted input order; recursive
input discovery does not follow symlinks; archive paths are confined to relative
sanitized components; and incomplete TAR shards remain `.part` files until close and
rename succeeds.

### RP-M1-03 classification boundary

The canonical item stream now has an immutable routing policy that distinguishes visible, deleted, associated, unknown, filtered, and readpst-skipped classes. It is intentionally below the command-line layer: `-D`, `-t`, output-profile selection, and adapter scheduling remain RP-M3-03 work. No CLI option is implied by the presence of a routing status in `data/items.jsonl`.

### RP-M6-02 crypt credential boundary

The pinned libpst NDB crypt methods do not accept a user password: method 1 is a fixed
substitution permutation and method 2 derives its salt from the data-block ID. PSTD
therefore exposes crypt method and decode status in capability/payload evidence without
adding a misleading password flag or wrong-password result. Unknown methods and
semantically undecodable encrypted payloads remain explicit unsupported/failed evidence.

## Command surface

| readpst option | Upstream behaviour | PSTD status | Required closure |
|---|---|---|---|
| `-h`, `-V` | Help and version output. | **Partial** | Keep the existing commands, publish a parity-oriented help surface, and expose the supported input/output feature set. |
| `-o <dir>` | Change to an output directory after opening the PST. | **Implemented** for structured output | Preserve safe path handling and make every output adapter use the same output-root contract. |
| `-d <file>`, `-L <level>`, `-q` | Debug file, debug level, or errors-only console output. | **Partial** | Add bounded structured diagnostics with equivalent severity filtering and file/JSONL destinations. |
| `-j <n>` | Parallel folder/message processing, with an auto-sized default. | **Partial** | Add bounded deterministic parallelism; output order and error aggregation must not depend on scheduling. |
| `-w` | Overwrite existing mbox/separate outputs. | **Partial** | Define overwrite/skip/fail semantics for every adapter and record the decision in the run manifest. |
| `-C <charset>` | Default character set for items without an explicit charset. | **Gap** | Add a per-run fallback charset option and preserve the selected source/target charset in records. |
| `-8` | Prefer UTF-8 bodies when an UTF-8 version exists. | **Partial** | Add explicit body-selection policy rather than always assuming UTF-8; preserve original bytes when conversion is lossy or unavailable. |
| `-D` | Include deleted items. | **Gap** | Traverse and emit deleted content by default-excluded policy, with source flags and a deterministic include option. |
| `-t[eajc]` | Filter output to email, appointment, journal, and contact classes. | **Partial** | Routed source identities now filter every named output projection without hiding canonical skipped counts; broad mixed-folder differential evidence remains. |
| `-a <exts>` | Keep only attachments whose filename extension is in a comma-separated allow-list. | **Partial**: mailbox and MSG/EML MIME projections now apply the same normalized case-insensitive allow-list as separate attachment publication while canonical records retain filtered decisions. | Full cross-input differential coverage remains. |
| `-b` | Do not emit the decompressed RTF body as `rtf-body.rtf`. | **Partial** | Make RTF preservation a policy choice in structured output and every MIME/file adapter. |
| `-c[v\|l]` | Emit contacts as vCard or a simple `name <address>` list. | **Gap** | Implement both contact adapters from one typed contact record. |

## Output modes

The mode flags are mutually exclusive in readpst. PSTD should expose equivalent named output profiles rather than reproducing ambiguous combinations of short flags.

| readpst mode | Observable result | PSTD status |
|---|---|---|
| default | One mbox-style file per PST folder and reduced item type, with multiple messages separated by mbox `From ` lines. | **Partial**: `mbox` emits deterministic email streams over canonical mail records; reduced typed streams and full differential corpus remain. |
| `-r` | A directory tree matching the PST folder tree; each directory contains an mbox file such as `mbox`, `calendar`, `contacts`, or `journal`. | **Partial**: `recursive_mbox` emits safe folder trees and explicit skipped/unavailable decisions; typed side streams remain downstream. |
| `-S` | A directory tree with numbered individual message files and separate binary attachment files. | **Partial**: `separate` emits numbered RFC 822 files plus resolved non-empty `<message-file>-<filename>` attachments; full differential coverage remains. |
| `-M` | MH/rfc822 individual message files without output extensions. | **Partial**: `mh` emits numbered files without mbox separators; full readpst corpus remains. |
| `-e` | MH/rfc822 individual message files with extensions, normally `.eml`/`.vcf`/`.ics`. | **Partial**: `eml` emits numbered `.eml` files; typed non-mail extensions remain downstream. |
| `-m` | The `-e` result plus `.msg` files. | **Partial**: `msg` emits deterministic CFB/OLE `.msg` files and `.eml` companions from canonical records; independent property/recipient/attachment round trips pass, while full named-property and embedded-message breadth remains explicit. |
| `-k` | KMail directory layout, including folder mbox names and index invalidation behaviour. | **Partial**: `kmail` emits safe `.<folder>.directory/<folder>.mbox` entries and explicit index policy; import/read coverage remains. |
| `-u` | Thunderbird recursive mode plus `.type` per folder and `.size` counts. | **Partial**: `thunderbird` emits recursive mbox, explicit `.type` source-status JSON, `.size` counts, and typed non-mail files; exact import compatibility remains. |
| `-c[v]` | vCard output with contact fields, notes, categories, and RFC 2426 escaping. | **Gap** |
| `-c[l]` | Simple contact email list. | **Gap** |
| appointment/journal selection | iCalendar or vJournal records, including recurrence and alarms where available. | **Gap** |

## Filename and stream semantics to preserve

The output adapters need explicit tests for behaviours that are easy to lose in a rewrite:

- folder and item names are converted to UTF-8 and sanitized before becoming paths;
- existing names are made unique unless overwrite is selected;
- separate message numbering starts at 1 and is local to the folder;
- attachment names prefer the long filename and fall back to the 8.3 filename;
- duplicate attachment names receive a deterministic numeric suffix;
- separate attachment files use `<message-file>-<filename>` and never publish filtered, unresolved, embedded, or zero-length payloads;
- an unnamed attachment receives a stable generated name;
- empty output files are removed by readpst and must instead be represented as skipped/unavailable records when the payload cannot be proven;
- mbox output uses `From ` separators and mboxrd escaping, while one-message-per-file output does not add a separator;
- output type filtering must support folders containing mixed item types;
- output counts must distinguish stored, emitted, skipped, unavailable, and failed items.

## Recommended PSTD shape

Keep the current structured TAR/JSONL result as the canonical extraction contract and add output adapters over validated records:

```text
PST/OST -> bounded parser -> typed records + raw artefacts
                              ├─ canonical TAR/JSONL
                              ├─ mbox / recursive mbox
                              ├─ MH / EML / separate files
                              ├─ KMail / Thunderbird layouts
                              ├─ vCard / vCalendar / vJournal
                              └─ MSG/OLE (Rust-native writer + independent reader)
```

An adapter must never reparse the PST or invent a value that is missing from the typed evidence. A requested output that cannot be constructed must produce a scoped unavailable result, not a plausible but incomplete file.

## Post-RP-M7 output parity expansion

The production runner now separates canonical evidence from output selection. When a
typed `-t[eajc]` filter is requested, routed `ItemEnvelope` identities select the
matching mailbox/MSG messages and typed contact, appointment, journal, and Thunderbird
records. The complete `data/items.jsonl`, routing counts, messages, and payload evidence
remain in the archive, so a projection cannot erase filtered source content.

The normalized `-a` allow-list is applied to generated MIME parts and Rust-native MSG
compatibility EML as well as separate binary files. Filtered attachments produce
explicit adapter decisions; canonical `AttachmentRecord` values and raw payloads are
unchanged. This is an output-equivalent implementation slice with focused synthetic
coverage, not a full readpst differential claim.

## Planned implementation — `RP-01`

### Readpst logic reviewed

The option parser in `readpst.c::main` maps `-a`, `-b`, `-C`, `-c[v|l]`, `-D`, `-d`, `-e`, `-j`, `-k`, `-L`, `-M`, `-m`, `-o`, `-q`, `-r`, `-S`, `-t[eajc]`, `-u`, `-V`, `-w`, and `-8` into process-wide settings. `-a` stores a NUL-separated, case-insensitive extension list; `-j` forks workers and children reopen the PST; `-M`, `-e`, and `-m` select separate-file extensions; `-r`, `-S`, `-k`, and `-u` select directory/output behaviour. `create_enter_dir`, `mk_recurse_dir`, `mk_separate_dir`, `mk_separate_file`, and the close functions implement names, collision suffixes, empty-file deletion, counts, KMail index invalidation, and Thunderbird sidecars. The regression script demonstrates the supported combinations and the operational expectation that one run can exercise recursive, contact, filter, charset, and worker policies.

### Planned PSTD model

Add an explicit `ReadpstProfile` layered over the existing `ExtractConfig`:

```text
InputPolicy { fallback_charset, prefer_utf8, include_deleted, include_associated }
ItemFilter { email, appointment, journal, contact }
OutputProfile { canonical, mbox, recursive_mbox, mh, eml, separate, kmail,
                thunderbird, vcard, contact_list, icalendar, vjournal, msg }
AttachmentPolicy { extension_allow_list, emit_separate_files, emit_synthetic_rtf }
ExecutionPolicy { jobs, overwrite, collision, diagnostics, limits }
```

The CLI parser should translate legacy flags into this typed configuration and reject ambiguous combinations with a stable error. The canonical JSONL/TAR profile is always available; legacy profiles are adapters under `src/output/` and share `src/output/paths.rs`, `src/output/ids.rs`, the manifest writer, and the typed envelope from `RP-03`.

### Implementation flow

1. Parse options without mutating global state. Normalize extensions once (`.DOC`, `doc`, and `docx` compare case-insensitively) and preserve the user’s original spelling in diagnostics.
2. Open, classify, and index the input before constructing the output root. Unlike readpst’s `chdir`, pass an absolute or workspace-relative root to every adapter.
3. Discover and type all items, including filtered/deleted/associated items, before applying a profile. This keeps canonical counts independent of output selection.
4. Schedule folder/item work with a bounded worker pool. Assign each job a stable `(folder_id, item_id, ordinal)` key; collect results by key so `jobs=1` and `jobs=N` produce identical JSONL, paths, and hashes.
5. For mailbox profiles, stream messages in source order through an mbox writer that performs mboxrd escaping and records each message offset/hash. For separate profiles, use per-folder ordinal numbers starting at 1 and route extensions from the typed item kind.
6. For KMail and Thunderbird, emit the folder tree plus deterministic sidecars from the same count object. Sidecars must expose stored, filtered, skipped, unavailable, and failed counts instead of only the readpst-compatible total.
7. Apply overwrite/collision policy atomically. `skip`, `fail`, `replace`, and `suffix` must be explicit and recorded per path.
8. Close the run by writing the manifest and adapter summary even when individual outputs fail. A profile failure must not erase canonical evidence.

### Improvements over readpst

- Keep the profile composable and validate combinations instead of relying on mutually exclusive global mode flags.
- Use the existing path sanitizer plus a stable source-ID suffix; never use readpst’s eight-digit collision heuristic as the only identity.
- Preserve empty/unavailable streams as manifest records rather than deleting them and losing the reason.
- Keep all output paths under the requested root, with atomic temporary files and no process-wide current-directory changes.
- Treat `-a` filtering as a payload projection; attachment metadata and filtered status remain in canonical output.
- Make diagnostics structured and bounded, with JSONL and human-readable renderings selected independently of extraction.

### Issue-ready acceptance

`RP-01` issues should be split into `RP-01A` (configuration/flag translation), `RP-01B` (deterministic scheduler), `RP-01C` (mailbox/separate adapters), `RP-01D` (KMail/Thunderbird), and `RP-01E` (diagnostics and overwrite policy). Each issue needs a profile fixture and assertions for:

- exact option-to-policy translation, including invalid combinations;
- identical canonical results at `jobs=1` and a higher bounded worker count;
- path confinement, collision, and atomic publish behaviour;
- mbox separators/escaping versus one-message-per-file output;
- sidecar counts reconciled with canonical item statuses;
- rerun behaviour for every overwrite mode.

The affected documents are [folders and item types](03-folders-and-item-types.md), [attachments](06-attachments.md), [storage outputs](09-storage-and-interoperability.md), [the matrix](10-parity-matrix.md), and [the roadmap](11-roadmap-and-acceptance.md).

## RP-M3-03 typed policy delivery

PSTD now translates the readpst-visible policy surface into a serializable
`ReadpstPolicy` before extraction. The policy records canonical versus named legacy
output profiles, fallback charset and UTF-8 preference, deleted/associated visibility,
one typed `-t` family filter, normalized attachment extensions, synthetic RTF policy,
bounded jobs, diagnostics, collision, and overwrite settings. The canonical path
applies the visibility/type filter to `data/items.jsonl` routing statuses while retaining
source IDs and raw evidence references; it also records the policy in the run manifest.

RP-M5-01 and RP-M5-02 now implement `mbox`, `recursive_mbox`, `mh`, `eml`, `separate`,
and `kmail` as projections over canonical records. Thunderbird and MSG remain fail closed
with a stable `RPCLI_UNSUPPORTED_OUTPUT_PROFILE` result until their dedicated adapters are
merged. Separate attachment publication consumes normalized extension policy and keeps
filtered/unavailable decisions explicit.
Invalid item-type combinations, attachment extensions, charset names, diagnostic levels,
collision policies, and job bounds likewise return explicit `RPCLI_*` configuration
errors. Profile selection therefore cannot silently fall back to canonical output.
