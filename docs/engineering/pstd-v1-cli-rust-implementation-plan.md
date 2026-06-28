# PSTD v1 CLI and Rust Implementation Plan

## Purpose

Define how the first implementation milestone should shape the Rust and Python code without implementing PST parsing yet.

## Implementation principles

- Rust owns extraction and archive writing.
- Python owns orchestration and operator convenience.
- Python must not parse PST internals.
- Rust must not call Python per message.
- Output contract writers must not know PST internals.
- PST reader modules should expose internal extraction structs, not JSON directly.
- Future Snowflake, React, Bun, Vite, search, semantic search, tagging, and graph features should consume v1 outputs rather than entering the v1 hot path.

## Proposed Rust layout

```text
pstd/
  Cargo.toml
  src/
    main.rs
    lib.rs
    cli.rs
    config.rs
    error.rs
    progress.rs

    pst/
      mod.rs
      reader.rs
      header.rs
      primitives.rs
      folders.rs
      messages.rs
      attachments.rs

    output/
      mod.rs
      archive.rs
      tar_writer.rs
      jsonl_writer.rs
      paths.rs
      ids.rs
      manifest.rs
      metadata.rs
      bodies.rs
      attachments.rs
      errors.rs
      summary.rs

    extract/
      mod.rs
      runner.rs
      records.rs

  tests/
    cli_smoke.rs
    output_contract.rs

  fixtures/
    README.md
```

This follows the repo skill reference while allowing future PST layers to expand into NDB, LTP, property contexts, table contexts, MAPI mapping, and address resolution modules.

## Rust module responsibilities

### `main.rs`

Binary entrypoint only.

### `cli.rs`

Parse user commands and options. Do not perform extraction directly.

### `config.rs`

Resolve validated runtime configuration from CLI args.

### `error.rs`

Shared error types and structured serializable error records.

### `progress.rs`

Structured progress events for console output and JSONL progress logs.

### `pst/`

Future PST traversal and extraction boundary. M1 may include placeholder traits or no-op implementations only.

### `extract/`

Coordinates extraction flow from reader to output writers. In M1 this may use placeholder records to validate output shape.

### `output/`

Owns archive contract writers, JSONL serialization, body/attachment path writing, manifest, summary, and errors.

## Initial CLI shape

Preferred command:

```text
pstd extract --input <pst-file-or-directory> --output <output-directory>
```

Useful options:

```text
--continue-on-error
--overwrite
--manifest-only
--archive-format tar
--data-format jsonl
--tar-shard-size-mb <mb>
--progress auto|plain|jsonl|none
--log-level error|warn|info|debug|trace
--profile fast|balanced|audit|debug
```

Open naming decision:

- The repo currently says PSTD. The binary could be `pstd`.
- Earlier planning notes used `pstfast`. If the project should stay aligned to the repo name, prefer `pstd`.

## Future CLI shape

Later milestones may add:

```text
pstd inspect --input <pst-file>
pstd validate-output --input <archive-or-output-root>
pstd benchmark --input <pst-file-or-directory> --output <report-path>
```

## Exit codes

```text
0: completed without errors
1: completed with one or more failed or partial items
2: invalid arguments or configuration
3: source could not be opened
4: output could not be written
5: fatal PST parse failure for requested input
```

## Python layout

```text
python/
  pyproject.toml
  src/
    pstfast/
      __init__.py
      __main__.py
      cli.py
      config.py
      runner.py
      progress.py
      reports.py
```

Open naming decision:

- If the user-facing package should align to the repo, use `pstd` instead of `pstfast`.
- If using `pstfast`, document why the package name differs from the repo name.

## Python responsibilities

- Provide a convenient wrapper around the Rust binary.
- Validate paths before invoking Rust where useful.
- Stream Rust progress output.
- Prepare future batch orchestration.
- Provide operator-facing summaries.

## Python non-responsibilities

- PST binary parsing.
- Message extraction.
- Attachment extraction.
- TAR hot-path writing.
- Per-message transformation.

## M1 placeholder flow

M1 can validate the archive contract without parsing PST files:

```text
CLI args
  -> Config
  -> placeholder extraction runner
  -> TAR shard writer
  -> JSONL writer
  -> summary/errors/progress
```

This gives later PST parser milestones a tested output boundary.

## M2+ real extraction flow

```text
CLI args
  -> Config
  -> PST mmap reader
  -> PST parser layers
  -> internal extraction structs
  -> output writer
  -> TAR shards + JSONL + bodies + attachments
```

## Performance notes

- Avoid loading whole PST files into memory.
- Avoid writing millions of loose files.
- Avoid base64 attachments in JSON.
- Avoid crossing Rust/Python boundary per message.
- Stream JSONL and TAR entries.
- Keep compression off by default until benchmarked.
- Make validation optional so maximum-speed runs can disable expensive checks.

## Deferred parser modules

Future PST parser milestones should add:

```text
pst/mmap_reader.rs
pst/header.rs
pst/primitives.rs
pst/bbt.rs
pst/nbt.rs
pst/block.rs
pst/ltp.rs
pst/heap.rs
pst/property_context.rs
pst/table_context.rs
mapi/properties.rs
mapi/recipients.rs
mapi/threading.rs
mapi/addresses.rs
```

## Suggested implementation commands for later validation

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
pstd --help
pstd extract --help
python -m pstfast --help
```

If the package or binary names differ, update these commands in the PR before claiming validation.
