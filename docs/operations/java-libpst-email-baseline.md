# Java-libpst email fixture baseline

This document records PSTD's measured email-focused baseline for the pinned public fixture `tests/fixtures/upstream/java-libpst-dist-list.pst`.

## Fixture identity

- SHA-256: `c86841da106036b5abe5a2141dc7644cbb2bf8b504873515eb35a2efeb8c28ac`
- Provenance: java-libpst upstream fixture, retained in PSTD with its pinned hash
- Measurement workflow: `.github/workflows/java-libpst-email-baseline.yml`
- First locked workflow run: `29861493130`
- Artifact digest: `sha256:d66d90089e215040b4c0dd78842d641787ad856eeba1c11ffb23cc36733bc51c`

External implementations may be used to inventory or compare this fixture, but PSTD's acceptance result below comes from PSTD's own Rust extraction and EML paths.

## Deterministic PSTD result

Two complete extraction runs produced semantically identical structured archives. Two EML runs produced identical empty output directories.

| Measurement | Exact result |
|---|---:|
| Extraction exit code | `0` |
| EML exit code | `1` (`eml_files_emitted=0`) |
| Folders | `25` |
| Message metadata records | `9` |
| Body records | `12` |
| Recipient records | `0` |
| Attachment metadata records | `22` |
| Materialised attachment payloads | `0` |
| EML files | `0` |
| `IPM.Note*` classes proven | `0` |

Three non-empty text payload files were materialised by the extraction archive:

| Path | Bytes | SHA-256 |
|---|---:|---|
| `bodies/msg_1ff382bb60543085.txt` | `23` | `3ea1b807ae5efd0cc0475450391ba540c48366afe8b9718a8af3655e53ee585f` |
| `bodies/msg_305bf74f071d49c5.txt` | `30` | `63ce530127bb594011cb249a9c05b14ca215008dce20262a88fb6d5fb2791c6f` |
| `bodies/msg_5c587e72d15fc581.txt` | `25` | `fdde548cf6f81a28076253ddd12b3637bfa477178fec3a8f98f1c3b457e1f4db` |

The nine message records do not currently expose a validated `PidTagMessageClass`; the baseline summary therefore records all nine as `None`. Subjects and folder placement indicate contacts, a distribution list, free/busy data, and associated-view metadata rather than proven email messages. Those hints are not sufficient to classify the objects or emit EML.

## Fail-closed interpretation

This fixture does **not** prove a new email-to-EML capability. In particular:

- attachment metadata without bounded payload ownership is not materialised;
- body payloads are not promoted to email without a validated mail class and required message evidence;
- contacts, distribution lists, free/busy data, and associated metadata are not forced into EML;
- absent message classes are not inferred from subjects, folders, or external-parser labels.

The `pstd-eml` exit code `1` and zero emitted files are therefore the correct current result, not silent data loss disguised as success.

## Compatibility status

| Capability | Status on this fixture |
|---|---|
| Unicode container traversal | Proven for the measured nodes |
| Folder discovery | Proven: 25 folders |
| Metadata record discovery | Proven: 9 records |
| Plain-text payload recovery | Partially proven: 3 non-empty payload files |
| Authoritative mail classification | Not proven |
| Recipient extraction | Not proven |
| Attachment payload materialisation | Not proven |
| EML generation | Explicitly unavailable |
| Typed non-mail classification | Not implemented; objects remain unclassified metadata |

## Decision

No parser or MIME change is justified from this fixture alone. The next email-focused compatibility fixture should contain independently evidenced `IPM.Note*` messages with either:

1. another by-value attachment layout or multiple exactly owned attachments; or
2. an inline attachment whose `PidTagAttachContentId` exactly matches an HTML `cid:` reference.

That fixture must be immutable, redistributable, pinned by byte length and SHA-256, independently inventoried, and then validated through PSTD before implementation changes are accepted.
