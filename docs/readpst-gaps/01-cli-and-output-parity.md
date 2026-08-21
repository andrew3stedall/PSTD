# CLI and output-mode parity

`readpst` exposes a single command with a compact flag set. PSTD currently exposes `inspect`, `extract`, `batch`, and `version`, with structured TAR/JSONL output. The structured interface is a good foundation, but it does not yet provide an equivalent for the legacy output modes.

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
| `-t[eajc]` | Filter output to email, appointment, journal, and contact classes. | **Gap** | Add typed item filtering at extraction and output-adapter layers without hiding skipped counts. |
| `-a <exts>` | Keep only attachments whose filename extension is in a comma-separated allow-list. | **Gap** | Implement case-insensitive extension filtering while retaining metadata records for filtered attachments and an explicit status. |
| `-b` | Do not emit the decompressed RTF body as `rtf-body.rtf`. | **Partial** | Make RTF preservation a policy choice in structured output and every MIME/file adapter. |
| `-c[v\|l]` | Emit contacts as vCard or a simple `name <address>` list. | **Gap** | Implement both contact adapters from one typed contact record. |

## Output modes

The mode flags are mutually exclusive in readpst. PSTD should expose equivalent named output profiles rather than reproducing ambiguous combinations of short flags.

| readpst mode | Observable result | PSTD status |
|---|---|---|
| default | One mbox-style file per PST folder and reduced item type, with multiple messages separated by mbox `From ` lines. | **Gap** |
| `-r` | A directory tree matching the PST folder tree; each directory contains an mbox file such as `mbox`, `calendar`, `contacts`, or `journal`. | **Gap** |
| `-S` | A directory tree with numbered individual message files and separate binary attachment files. | **Gap** |
| `-M` | MH/rfc822 individual message files without output extensions. | **Gap** |
| `-e` | MH/rfc822 individual message files with extensions, normally `.eml`/`.vcf`/`.ics`. | **Partial**: EML exists, but not as a general readpst-compatible adapter. |
| `-m` | The `-e` result plus `.msg` files. | **Gap** |
| `-k` | KMail directory layout, including folder mbox names and index invalidation behaviour. | **Gap** |
| `-u` | Thunderbird recursive mode plus `.type` per folder and `.size` counts. | **Gap** |
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
                              └─ MSG (only after a separately tested writer)
```

An adapter must never reparse the PST or invent a value that is missing from the typed evidence. A requested output that cannot be constructed must produce a scoped unavailable result, not a plausible but incomplete file.

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
