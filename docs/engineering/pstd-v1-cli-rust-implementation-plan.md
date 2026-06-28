# PSTD v1 CLI and Rust Implementation Plan

## Purpose

Define how the first implementation milestone shapes the Rust and Python code before PST parsing is added.

## Naming decision

The Rust binary and Python package are named `pstd`.

## Rust layout

```text
Cargo.toml
src/
  main.rs
  lib.rs
  cli.rs
  config.rs
  error.rs
  progress.rs
  engine/
    mod.rs
    runner.rs
    records.rs
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
    tar_writer.rs
    jsonl_writer.rs
    paths.rs
    ids.rs
    metadata.rs
    summary.rs
tests/
  cli_smoke.rs
  output_contract.rs
```

## CLI shape

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

## Python layout

```text
python/
  setup.py
  src/
    pstd/
      __init__.py
      __main__.py
      cli.py
```

## M1 placeholder flow

```text
CLI args
  -> Config
  -> placeholder engine runner
  -> TAR shard writer
  -> JSONL writer
  -> summary/status/progress records
```

## M2+ real flow

```text
CLI args
  -> Config
  -> PST reader/parser layers
  -> internal records
  -> output writer
  -> TAR shards + JSONL + bodies + attachments
```

## Validation commands

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
pstd --help
pstd extract --help
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
```
