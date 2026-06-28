# PSTD Module Boundaries Reference

## Purpose

Keep implementation work clean as the project grows.

## Boundary rules

- CLI code should parse user intent, not perform extraction directly.
- PST reading code should expose extracted records, not write files directly.
- Output code should know the output contract, not PST internals.
- Error logging should be structured and reusable across PST, message, body, and attachment extraction.
- Progress reporting should consume events rather than inspecting internals.

## Suggested flow

```text
CLI args
  -> Config
  -> Extractor
  -> Output writers
  -> Manifest, summary, errors, messages, attachments
```

## Data model boundary

Use internal structs for extraction results. Convert those structs to stable output JSON at the output layer.

## Future integration boundary

Future Snowflake, search, React, or API features should consume v1 outputs. They should not require PST parsing logic to be embedded into web or warehouse components.
