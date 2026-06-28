# PSTD CLI Design Reference

## Purpose

Define the first CLI shape for PSTD implementation planning.

## Initial command

```text
pstd extract --input <pst-file-or-directory> --output <output-directory>
```

## Useful options

```text
--input <path>              PST file or directory of PST files
--output <path>             Output root directory
--continue-on-error         Continue extracting when recoverable errors occur
--overwrite                 Allow writing into an existing output directory
--manifest-only             Produce inventory and manifest without extracting full bodies
--log-level <level>         error, warn, info, debug, trace
--progress <mode>           auto, plain, jsonl, none
```

## Progress output

Default progress should be human-readable. A later `jsonl` mode can support automation.

Example plain progress:

```text
PSTD extract started
Source: input/example.pst
Output: output/example
Folders: 12 discovered
Messages: 995 extracted, 5 failed
Attachments: 247 extracted, 3 failed
Status: completed_with_warnings
```

## Exit codes

- `0`: completed without errors.
- `1`: completed with one or more failed items.
- `2`: invalid arguments or configuration.
- `3`: source could not be opened.
- `4`: output could not be written.

## UX rules

- Always print where the manifest and error log were written.
- Do not hide partial failures.
- Do not require Snowflake or web UI assumptions for v1 CLI extraction.
- Make reruns deterministic where possible.
