# Linux-native ANSI PST fixture generation

## Decision

PSTD must remain buildable, testable, and reproducible on Linux using Rust. Windows, Outlook, COM, PowerShell, proprietary SDKs, and manual mailbox tooling are not acceptable fixture dependencies.

The previous Outlook-based generation path has been removed.

## Purpose

Vertical 40 still requires a real version-14 or version-15 ANSI PST before ANSI BBT/NBT traversal can be enabled. The fixture-generation work therefore becomes a bounded Rust engineering task: implement only enough deterministic ANSI PST writing to produce a small controlled corpus, then use PSTD itself to validate the resulting bytes fail closed before parser support is expanded.

This is fixture infrastructure, not a general PST writer and not evidence of broad ANSI compatibility.

## Required command

The accepted end state must run on Linux with a command of this form:

```bash
cargo run --locked --bin pstd-generate-ansi-fixture -- \
  --output target/fixtures/pstd-ansi-baseline.pst
```

The generator must use only Rust dependencies permitted by the repository licence and lockfile. It must run in CI without network access after dependencies are fetched.

## Controlled corpus

The first generated PST should contain the smallest evidence set needed for parser development:

- root folder;
- `/Synthetic Mail` and `/Synthetic Mail/Nested`;
- one plain-text mail with To, Cc, and Bcc rows;
- one mail with independent plain and HTML bodies;
- one by-value text attachment;
- `/Typed Non-Mail` containing one contact explicitly classified as non-mail.

All strings, addresses, timestamps, identifiers, and payload bytes must be fixed constants. No host names, users, clocks, randomness, environment values, or private data may enter the output.

## Determinism and format boundaries

The generator must:

- emit `!BDN` magic and `SM` client signature;
- emit NDB version 14 or 15;
- use ANSI-specific 32-bit BIDs and file offsets;
- use bounded, checked offset arithmetic;
- produce byte-identical output across repeated Linux runs;
- reject duplicate identifiers, out-of-range offsets, overflows, unsupported object forms, and inconsistent page or block references;
- write pages and blocks through narrowly scoped builders rather than sharing Unicode parser assumptions implicitly;
- finish with exact byte length and SHA-256 output.

The initial writer scope is limited to the structures required by the controlled corpus. Unsupported writer features must return explicit errors rather than approximating Outlook behaviour.

## Test-first implementation sequence

1. Add unit tests for ANSI header, root structure, page trailers, BBT/NBT entries, block allocation, and deterministic identifiers.
2. Generate the same fixture twice and assert byte-for-byte equality and one pinned SHA-256.
3. Parse the fixture header with PSTD while ANSI traversal remains disabled and assert explicit unsupported-for-extraction status with zero emitted objects.
4. Add one bounded traversal slice at a time, preserving the generated fixture hash unless the writer contract intentionally changes.
5. Keep every approved Unicode fixture artifact unchanged.

## Admission manifest

When the Rust generator exists, commit a manifest containing at least:

```yaml
name: pstd-ansi-baseline.pst
format: Microsoft PST ANSI
ndb_version: 14
source: controlled Rust generator
command: cargo run --locked --bin pstd-generate-ansi-fixture -- --output target/fixtures/pstd-ansi-baseline.pst
repository_commit: <full commit SHA>
platform: linux
content: controlled synthetic data only
licence: CC0-1.0
byte_length: <exact integer>
sha256: <64 lowercase hexadecimal characters>
```

## Fail-closed boundaries

Do not enable ANSI extraction when:

- the Rust generator cannot reproduce identical bytes on Linux;
- the generated file does not pass independent structural checks;
- provenance, hash, size, format version, or licence is missing;
- parser traversal requires guessing page widths, entry widths, offsets, BIDs, or object ownership;
- malformed or ambiguous references are silently skipped;
- the controlled fixture would be mistaken for representative ANSI corpus coverage.

## Current status

PSTD currently has variant-correct diagnostic ANSI header decoding only. The Linux-native Rust fixture writer does not yet exist, and ANSI traversal, folders, messages, recipients, bodies, attachments, non-mail objects, and EML remain unsupported.
