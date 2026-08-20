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
