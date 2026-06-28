# PSTD v1 Output Contract Reference

## Purpose

Define the stable output shape that PSTD planning and implementation should preserve.

## Output directory shape

```text
<output-root>/
  manifest.json
  summary.json
  errors.jsonl
  folders.json
  messages/
    <message-id>/
      message.eml
      metadata.json
      body.txt
      body.html
      attachments/
        <attachment-id>-<safe-filename>
```

## Required message metadata

Each extracted message should preserve, where available:

- Stable generated message id.
- Source PST path.
- Source folder path.
- Subject.
- Sender.
- Recipients: to, cc, bcc.
- Sent time.
- Received time.
- Conversation or thread identifiers.
- Internet message id.
- In-reply-to and references headers.
- Attachment count.
- Extraction status.
- Error references.

## Body outputs

- Preserve plain text body when available.
- Preserve HTML body when available.
- Do not invent missing body representations.
- If a body cannot be extracted, record the reason in `errors.jsonl`.

## Attachment outputs

- Extract attachments into the message attachment folder when available.
- Use safe deterministic filenames.
- Preserve original filename in metadata.
- Record skipped or failed attachments in `errors.jsonl`.

## Manifest outputs

`manifest.json` is the run-level index. It should describe the source, tool version, run timing, totals, output layout, and generated artefacts.

## Error outputs

`errors.jsonl` is append-friendly and should contain one JSON object per warning or error.

## Stability rules

- Output paths should be deterministic for the same source data and configuration.
- Message ids should avoid collisions.
- Metadata field names should remain stable once released.
- Future Snowflake loading should not require reparsing raw PST files when v1 outputs exist.
