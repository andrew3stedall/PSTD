# PSTD Rust Project Structure Reference

## Purpose

Give implementation work a consistent initial Rust layout.

## Suggested initial layout

```text
pstd/
  Cargo.toml
  src/
    main.rs
    cli.rs
    config.rs
    error.rs
    lib.rs
    pst/
      mod.rs
      reader.rs
      folders.rs
      messages.rs
      attachments.rs
    output/
      mod.rs
      manifest.rs
      metadata.rs
      eml.rs
      bodies.rs
      attachments.rs
      errors.rs
      summary.rs
    progress.rs
  tests/
    cli_smoke.rs
    output_contract.rs
  fixtures/
    README.md
```

## Module intent

- `main.rs`: binary entrypoint only.
- `cli.rs`: argument parsing and command definitions.
- `config.rs`: resolved runtime configuration.
- `error.rs`: shared error types.
- `pst/`: PST traversal and extraction boundary.
- `output/`: output contract writers.
- `progress.rs`: progress reporting.

## Implementation rules

- Keep PST parsing separate from output writing.
- Keep CLI parsing separate from extraction logic.
- Use deterministic output paths.
- Prefer structured errors over string-only errors.
- Keep future Python, React, and Snowflake needs out of the Rust MVP unless the milestone explicitly includes them.

## First coding milestone

The first implementation milestone should establish:

- Cargo project.
- CLI entry point.
- Output directory creation.
- Placeholder manifest and summary writer.
- Deferred tests or smoke tests.
