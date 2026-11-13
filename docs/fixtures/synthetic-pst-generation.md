# Synthetic Unicode PST generation

## Purpose

`tools/synthetic_pst.rs` writes a Unicode PST of any size from a seed, together with a
manifest of what a correct reader must recover. It exists to measure extraction at
scale and to pin robustness behaviour. No committed fixture is larger than 2.3 MB, and
none contains deliberately damaged items.

All content is synthetic: reserved example domains, generated words, and pseudo-random
bytes derived from the seed. The same arguments always produce the same bytes.

## Boundary

This is a fixture-only emitter, not a general PST writer:

- It is written from the MS-PST layouts and shares no code with the PSTD reader, so a
  comparison of reader output with the manifest compares two implementations.
- It writes the header, NBT and BBT pages, data blocks with trailers and checksums,
  XBLOCK/XXBLOCK data trees, SLBLOCK subnode trees, multi-block heap-on-node, BTH,
  property contexts and table contexts.
- It does not write allocation maps, density lists or search structures. The output is
  not intended to be opened or repaired by Outlook.
- Data blocks are not encoded (`bCryptMethod` 0). ANSI output is out of scope; see
  `docs/fixtures/ansi-pst-generation.md`.

A generated file is evidence about the reader, not proof of compatibility with
mailboxes written by Outlook.

## Usage

```bash
rustc --edition=2021 -O --deny warnings tools/synthetic_pst.rs -o /tmp/synthetic-pst
/tmp/synthetic-pst --messages 20000 --seed 7 --manifest /tmp/mailbox.json /tmp/mailbox.pst
```

The emitter refuses to overwrite an existing output. Tests include the same file as a
module (`tests/synthetic_pst.rs`) and call `generate` directly.

## Mailbox content

Folders: Top of Personal Folders, Search Root, Deleted Items, Inbox, Drafts, Sent
Items, Archive, Archive/2023, a nested folder with a non-ASCII name, an empty folder,
Contacts and Calendar. Each folder has a hierarchy table, a contents table and an
associated-contents table.

Regular messages carry sender, recipients, timestamps, a plain-text body, sometimes an
HTML body and sometimes one or two attachments. Drafts have the unsent flag and no
sender or recipients. Messages in Deleted Items are ordinary mail.

The first 24 messages, and every sixteenth message after that, take one special
scenario in rotation:

| Scenario | What it exercises |
|---|---|
| `no_subject`, `empty_body`, `unicode_subject`, `long_subject`, `html_only` | Absent and unusual text properties |
| `large_body` | A 60 KB body stored in a subnode data tree |
| `many_recipients` | 300 recipients; table rows stored in a subnode |
| `no_recipients` | A received message without a recipient table |
| `embedded_message` | An attachment that is itself a message |
| `duplicate_message_id`, `orphan_message` | Repeated message id; a message in no contents table |
| `zero_length_attachment`, `large_attachment`, `many_attachments` | Attachment size extremes and 25 attachments |
| `attachment_missing_block` | Attachment data block absent from the block index |
| `attachment_truncated_tree` | Data tree whose last block is absent |
| `attachment_garbage_properties` | Attachment property context replaced by random bytes |
| `attachment_checksum_mismatch` | One data byte differs from the block checksum |
| `attachment_dangling_row` | Attachment table row with no attachment node |
| `message_missing_block`, `message_garbage_properties` | A whole message that cannot be read |
| `dangling_contents_row` | Contents-table row with no message node |
| `contact`, `appointment` | Non-mail items |

Every damaged-attachment message also carries an intact attachment, so containment is
checkable: the message, its body and the intact attachment must still be extracted.

## Manifest

`--manifest` writes one JSON document: seed, file length, folders, dangling contents
rows, and per message its node id, folder, scenario, class, subject, body length and
FNV-1a checksum, flags (draft, deleted, in contents table, recoverable), recipient
count, and per attachment its filename, size, FNV-1a checksum and fault.

## Validation status

- `tests/synthetic_pst.rs` generates 72 messages and checks byte determinism, scenario
  coverage, and PSTD extraction against the manifest: subject, class, message id,
  folder, draft flags, text body bytes, HTML presence, recipient count and intact
  attachment bytes. Unreadable messages must expose no decoded content.
- The test runs in the normal `cargo test --all` CI job.
- Not yet validated by an independent reader. No libpff or libpst build was available
  where the emitter was written, so agreement between the emitter and PSTD is the only
  evidence so far. A CI probe with `pffinfo`, `pffexport` and `readpst` is proposed in
  issue #621.

## Reader gaps found on first use

| Issue | Gap |
|---|---|
| #622 | Property values stored in subnodes, or empty, are emitted as four literal bytes (large bodies, empty and unresolvable attachments) |
| #623 | Recipient rows stored in a subnode are not read |
| #624 | Block checksums are not verified |
| #625 | A damaged attachment leaves no record |
| #626 | Folder nesting is not rebuilt |
| #627 | Output and memory footprint at 20,000 messages |

`tests/synthetic_pst.rs` excludes only the assertions covered by #622 and #623, through
its `*_GAPS` constants. Remove an entry when its issue is fixed.

## Scale measurement

20,000 messages, seed 7, development sandbox (2026-11-14):

| Measure | Value |
|---|---|
| Generation | 1.7 s, 51 MB peak RSS |
| PST size | 119,676,416 bytes |
| PSTD extraction | 10.8 s wall, 6.7 s user CPU |
| PSTD peak RSS | 1,508 MB |
| Output archive | 699,610,112 bytes |
