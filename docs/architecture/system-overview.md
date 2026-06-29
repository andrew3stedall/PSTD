# PSTD System Overview

## Purpose

Describe how PSTD is structured today and how the v1 pipeline is intended to evolve.

## Current v1 direction

PSTD is a Rust-first PST extraction tool. The v1 output target is structured TAR + JSONL data that can later be loaded into Snowflake, searched, reviewed, tagged, and used by graph/LLM workflows.

## Current pipeline

```text
CLI
  -> Config
  -> PST reader/parser layers
  -> Metadata extraction engine
  -> Output records
  -> TAR shards + JSONL + run summary + progress log
```

## Implemented layers

```text
pstd inspect
  -> bounded byte reader
  -> PST header parser
  -> BBT/NBT skeletons
  -> inspect summary

pstd extract --manifest-only
  -> bounded byte reader
  -> PST header parser
  -> BBT/NBT skeletons
  -> logical metadata layer
  -> folders/messages/status records
  -> structured TAR + JSONL output
```

## Future layers

```text
M4 Recipients/threading
  -> recipients.jsonl
  -> message_references.jsonl
  -> address resolution outputs

M5 Bodies/attachments
  -> bodies/*.txt/html
  -> attachments/<message-key>/...
  -> bodies.jsonl
  -> attachments.jsonl

M6 Batch orchestration
  -> checkpointing
  -> many-PST worker scheduling
  -> structured progress summaries

Future Snowflake/web/search
  -> consume v1 TAR + JSONL outputs
```

## Component responsibilities

| Component | Responsibility |
|---|---|
| Rust CLI | User-facing command surface. |
| Rust parser | Reads and interprets PST bytes. |
| Rust engine | Coordinates extraction and output writing. |
| Rust output layer | Writes stable TAR + JSONL artefacts. |
| Python wrapper | Operator convenience boundary only. |
| Docker scaffold | Local container execution. |
| Future Snowflake | Load/search extracted records, not parse PSTs. |
| Future web UI | Review, search, download, and tag extracted records. |

## Output shape

The v1 output contract is intentionally data-first:

```text
<output-root>/
  run_summary.json
  progress.jsonl
  archives/
    <pst-id>_000001.tar
```

Inside each TAR shard:

```text
_pstfast/
  summary.json
  manifest.jsonl
  errors.jsonl
  folder_inventory.jsonl
  run_config.json

data/
  folders.jsonl
  messages.jsonl
  recipients.jsonl
  message_references.jsonl
  bodies.jsonl
  attachments.jsonl
```

M3 currently writes the metadata subset. Later milestones populate recipients, references, bodies, and attachments.

## Design constraints

- Do not use external PST parsers.
- Do not write millions of loose files.
- Do not base64 attachment bytes into JSON.
- Preserve stable IDs for future joins.
- Record unsupported or partial extraction explicitly.
- Keep Snowflake and web UI out of parser milestones.

## Validation status

M1-M3 are implemented but local validation remains deferred. See [Validation Guide](../operations/validation-guide.md).
