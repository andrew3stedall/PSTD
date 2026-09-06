# Vertical 46: Reconstructible email content and attachment text

_Status: implemented on the ATT-11 branch; validation is required before merge._

## Objective

Make canonical extraction useful to a downstream EML builder and search pipeline
without requiring either system to reopen the source PST. Email content is emitted
as one JSONL row per message. Attachment bytes remain independent artefacts with
stable IDs and can be stored on disk or kept in the canonical archive.

## Output contract

`data/email_content.jsonl` contains:

- the complete serializable `MessageRecord` and selected header projection;
- ordered recipients and all body records;
- exact body payload bytes as base64, plus decoded/recovered RTF and HTML when
  validation makes those representations available;
- full attachment metadata references and an `attachment_ids` list; and
- an explicit reconstruction status.

The body bytes are embedded because they are part of the email itself. Attachment
bytes are not embedded in this row, avoiding a second large binary copy and keeping
attachment retrieval independently addressable.

## Attachment storage and retrieval

The canonical default remains `--attachment-storage archive`. The new modes are:

| Mode | TAR payload | Disk payload | Root manifest |
|---|---:|---:|---:|
| `archive` | yes | no | no |
| `disk` | no | yes | `attachments.jsonl` |
| `both` | yes | yes | `attachments.jsonl` |
| `none` | no | no | no |

Disk mode writes the existing safe `archive_path` below the extraction root. The
public retrieval method `retrieve_attachment_by_id(root, attachment_key)` reads the
sidecar manifest, rejects unsafe paths, and verifies the returned file's size and
SHA-256 against its metadata.

## Plain-text attachment projection

`--attachment-text office-pdf` writes `data/attachment_text.jsonl` with one row per
attachment. It preserves the source attachment ID, owning message ID, filename,
source path, source hash, and source size. Supported Office Open XML packages are
read in document/sheet/slide order; PDF text is extracted through the packaged
`pdftotext` runtime backend. The output is intentionally a reading projection, not
semantic classification or summarisation. Unsupported, malformed, unavailable, and
empty cases remain explicit statuses.

## Validation boundary

The ATT-11 workflow validates the public Tika DOCX fixture in archive, disk, and
both modes; exact body base64 round-tripping; stable attachment IDs and ownership;
disk retrieval; Office text extraction; and deterministic repeat output. PDF parsing
is covered with a generated well-formed text PDF and reports an explicit backend
unavailable status on runtimes without `pdftotext`.

Legacy `.doc`, `.xls`, and `.ppt` binary packages are not silently treated as OOXML.
They remain explicitly unsupported in the text projection while their raw bytes and
attachment metadata continue to follow the normal attachment path.
