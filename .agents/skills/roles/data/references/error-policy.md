# PSTD v1 Error Policy Reference

## Purpose

Define how PSTD should record extraction issues while continuing to recover as much useful data as possible.

## Default behaviour

PSTD should continue on error by default. A failure in one message, attachment, folder, or PST should not stop extraction of unrelated recoverable content.

## Error levels

Use stable severity values:

- `info`: useful run note.
- `warning`: recoverable issue, output may be incomplete.
- `error`: failed item, run can continue.
- `fatal`: run-level failure that prevents useful continuation for the current PST.

## Error scope

Use stable scope values:

- `run`
- `pst`
- `folder`
- `message`
- `attachment`
- `body`
- `metadata`

## Required fields

Every error record should include:

- `run_id`
- `source_pst`
- `scope`
- `severity`
- `code`
- `message`
- `source_folder_path` where known
- `message_id` where known
- `attachment_id` where known
- `timestamp_utc`
- `recoverable`

## Recovery notes

- Corrupted PSTs should produce partial output where possible.
- Unsupported item types should be logged rather than silently ignored.
- Failed attachments should not block message metadata extraction.
- Failed body extraction should not block raw metadata extraction.
- Failed metadata fields should be set to null or omitted according to the output contract, not guessed.
