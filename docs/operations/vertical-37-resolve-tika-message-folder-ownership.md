# Vertical 37: Resolve Tika message-folder ownership

_Last reviewed: 19 July 2026._

## Objective

Assign every top-level message in `tika-testPST.pst` to its authoritative physical folder from exact contents-table row keys, while leaving unsupported, absent, duplicate, or ambiguous membership unassigned.

## Root cause

Top-level messages were previously emitted under the synthetic root because metadata extraction did not consume physical contents-table membership. The first integration attempt also exposed an older discovery defect: `message_table.rs` classified PST NID types `0x12` and `0x13` as contents tables and `0x0d` as search contents. Those values identify recipient/search-index and hierarchy structures, so the real `0x0e` contents table was never admitted.

The corrected PST table types are:

| NID type | Meaning | Ownership authority |
|---:|---|---|
| `0x0d` | Hierarchy table | No |
| `0x0e` | Contents table | Yes, for normal messages |
| `0x0f` | Associated contents table | Yes, for associated messages |
| `0x10` | Search contents table | No |

## Implementation

- `src/pst/message_table_membership.rs` loads one linked table node, resolves its TC row-index, preserves exact row keys, and fails closed on truncated, zero, duplicate, or wrong-type rows.
- `src/engine/message_folder_ownership.rs` converts decoded physical rows into membership evidence and invokes the existing ownership reducer.
- `src/engine/metadata.rs` consumes the ownership report before constructing top-level messages. Resolved messages receive the exact folder key and path; unresolved or ambiguous messages receive explicit empty ownership fields instead of the synthetic root.
- `src/pst/message_table.rs` now uses the specification-aligned table NID types and excludes attachment and recipient tables from message-membership discovery.
- The temporary self-modifying workflow was deleted. All production changes are normal Rust source changes.

## Exact Tika result

The authoritative node `node_802e` decodes seven exact contents-table rows. All seven normal messages resolve once to:

```text
folder key:  folder_b6721eebbda60057
folder node: node_8022
folder path: /Début du fichier de données Outlook
folder items: 7
ownership:    message_table_membership_exact; table_type=contents_table
```

Exact owned message keys:

```text
msg_2a24cb244caffb52
msg_d7325f9579306ba3
msg_ce73d10db9eceadb
msg_33144a866cb58408
msg_5e70ad45c1c42472
msg_4bc9d7bd77cbe661
msg_c6163b9157944cc9
```

The recovered method-`5` child `msg_0ff529af59d373d5` remains a separately linked embedded message under the synthetic root record. Its relationship continues to come from the authoritative attachment link, not folder counts, NBT order, or contents-table inference.

The permanent Tika workflow now asserts all eight folder records, their Unicode names and paths, all seven top-level message keys and subjects, the exact owner key/path/status, and the separate embedded-child boundary.

## Preserved contracts

```text
messages:                       8
body records:                   10
body payload files / bytes:     8 / 279
recipient records:              9
attachment records:             2
attachment payloads / bytes:    2 / 12,315
EML files / bytes:              2 / 17,488
DOCX bytes:                     11,862
parent EML bytes:               17,035
child EML bytes:                453
messages JSONL bytes:           23,765
extraction TAR bytes:           237,056
total extraction-output bytes:  282,103
```

The message JSONL and archive sizes increase because the seven records now contain the decoded Unicode folder path and exact ownership status. Body, recipient, attachment, and EML bytes are unchanged.

## Fail-closed boundaries

- Hierarchy and search contents tables never establish ownership.
- An associated row cannot own a normal message, or vice versa.
- Truncated, zero, duplicate, and wrong-type row keys yield no members.
- Missing folder links, missing NBT candidates, decode errors, absent membership, and multiple physical rows remain explicit diagnostics.
- Folder item counts and discovery order are not used as fallback evidence.

## Validation

GitHub Actions compiled the source and passed the Rust tests, clippy with warnings denied, rustfmt, Python wrapper, Docker build, readable EML fixture, and readable RTF/HTML fixture. The Tika artifact from commit `bba8a54` confirmed the exact seven-row ownership result before the permanent semantic assertions and updated size snapshot were committed.

## Scope boundary

This is exact evidence for one approved Unicode PST. It does not establish broad PST compatibility, ANSI support, deleted-item recovery, multi-folder ownership across producers, or recursive embedded-message folder semantics.

## Next boundary

Validate independent plain-text, HTML, and RTF body selection on `tika-various-body-types.pst`, then add the first pinned public ANSI PST baseline.
