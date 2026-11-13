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
- It writes the mandatory nodes of the minimum mailbox: message store, name-to-id
  map, root folder, search management nodes, the table templates and the search
  folder object. Populated tables declare the template columns; cells the generator
  has no value for are absent.
- It does not write allocation maps, density lists or search contents. The output is
  not intended to be opened or repaired by Outlook, and tools that recover deleted
  space (`pffexport -m all`) reject it.
- Data blocks are not encoded (`bCryptMethod` 0). ANSI output is out of scope; see
  `docs/fixtures/ansi-pst-generation.md`.
- Size limits: one SLBLOCK per subnode tree (340 subnodes), two data-tree levels and
  65,536 heap blocks per node. 200,000 messages (1.2 GB) stay inside them.

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

`tests/synthetic_pst.rs` (in the normal `cargo test --all` job) checks:

- byte determinism, scenario coverage and manifest JSON validity;
- PSTD extraction of a 72-message mailbox against the manifest: one record per
  message, subject, class, message id, folder, draft flags, text body bytes, absence
  of a text body where there is none, HTML presence, embedded-message records and
  intact attachment bytes; unreadable messages expose no subject, sender, body,
  recipients or attachments;
- that both messages sharing a message id are extracted;
- that a 1,200-message mailbox (multi-level row indexes, rows in subnodes, multi-block
  heaps) yields one record with the right subject per message.

Independent readers, run by hand on 2026-11-14 against 0-, 1-, 72- and 400-message
files (libpff 20231205 `pffinfo`/`pffexport -m items`, libpst `readpst` 0.6.76 with
`-D -S`); not yet automated in CI (proposed workflow in issue #621):

| Check, 400 messages | readpst | pffexport |
|---|---|---|
| Opens the file | yes | yes |
| Folder nesting | correct | correct |
| Intact attachments byte-exact | 143 of 143 | 141 of 143 |
| Text bodies byte-exact | not compared | 389 of 393 |

The four bodies libpff does not export are orphan messages (it lists messages from
contents tables) and appointments (exported in another form). The two attachments are
on `attachment_dangling_row` messages, where libpff drops all attachments of the
message. libpff 20231205 also logs a non-fatal "unable to determine attachments" for
every message without an attachment table; this is its lookup of an absent subnode,
not a property of the file. readpst skips Deleted Items without `-D`.

This is evidence that the file is readable by two independent implementations. It is
not an Outlook compatibility claim.

## Reader gaps found

| Issue | Gap |
|---|---|
| #622 | Property values stored in subnodes, or empty, are emitted as four literal bytes (large bodies, empty and unresolvable attachments) |
| #623 | Table rows stored in a subnode are not read (many recipients; folder ownership in large folders) |
| #624 | Block checksums are not verified |
| #625 | A damaged attachment leaves no record |
| #626 | Folder nesting is not rebuilt |
| #627 | Output and memory footprint at 20,000 messages |
| #629 | Recipients are extracted only when a heuristic picks the recipient-type column |

`tests/synthetic_pst.rs` switches off only the assertions named by its gap constants
(`BODY_GAPS`, `INTACT_ATTACHMENT_GAPS`, `RECIPIENT_GAP`, `LARGE_FOLDER_GAP`,
`FOLDER_NESTING_GAP`, `DAMAGED_ATTACHMENT_GAP`). Remove an entry when its issue is
fixed.

## Scale measurement

20,000 messages, seed 7, development sandbox (2026-11-14):

| Measure | Value |
|---|---|
| Generation | 2.0 s, 60 MB peak RSS |
| PST size | 125,989,888 bytes |
| PSTD extraction | 9.8 s wall, 7.9 s user CPU |
| PSTD peak RSS | 1,482 MB |
| Output archive | 687,823,360 bytes |
