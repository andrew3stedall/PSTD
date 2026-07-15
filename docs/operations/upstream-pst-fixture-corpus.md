# Upstream PST fixture corpus

_Last reviewed: 16 July 2026._

## Purpose

PSTD's original public fixture is deliberately small and has been useful for validating one readable message, four recipients, plain text, RTF, recovered HTML, and deterministic EML assembly. It contains no attachment candidates and does not provide enough variation to validate broader PST compatibility.

This corpus adds three public, pinned PST files so extraction can advance through observable vertical milestones without using private client data. Each file has a different role. Do not treat the corpus as proof of general PST compatibility.

## Fixture inventory

| Local file | Size | SHA-256 | Primary purpose |
|---|---:|---|---|
| `tests/fixtures/upstream/tika-testPST.pst` | 2,302,976 bytes | `f2a6b1d2cad00f574e3d1c1211c4b1c854d6526caea77213adc3da92b7813ae3` | Multiple messages and folders, Unicode metadata, a documented DOCX attachment, and a legacy Exchange recipient address |
| `tests/fixtures/upstream/tika-various-body-types.pst` | 271,360 bytes | `24c5e6bbb8bf26a817c977283e40e7b69d2661fec0845abbe177f97efcb05fb0` | Multiple body representations and body-selection behaviour |
| `tests/fixtures/upstream/java-libpst-dist-list.pst` | 271,360 bytes | `c86841da106036b5abe5a2141dc7644cbb2bf8b504873515eb35a2efeb8c28ac` | Recurring appointments, recurrence exceptions, contacts, distribution lists, and non-mail MAPI objects |

Machine-readable hashes and sizes are stored beside the fixtures in `SHA256SUMS` and `FILE-SIZES.tsv`.

## Provenance

### Apache Tika `testPST.pst`

- Upstream repository: `apache/tika`
- Pinned commit: `63e22d08ef249cc73a6d02da7bc199fc3623a607`
- Upstream path: `tika-app/src/test/resources/test-data/testPST.pst`
- Source URL: `https://github.com/apache/tika/blob/63e22d08ef249cc73a6d02da7bc199fc3623a607/tika-app/src/test/resources/test-data/testPST.pst`
- Upstream project licence: Apache License 2.0

Apache Tika's regression tests document the following expected evidence:

- a 2,302,976-byte PST;
- multiple email-folder elements, including a non-ASCII French folder name;
- ten recursive metadata objects including the PST container;
- messages with Unicode names, including `Jörn Kottmann`;
- a nested `attachment.docx` whose content includes `This is a docx attachment.`;
- a legacy Exchange distinguished-name recipient beginning `/o=ExchangeLabs/ou=Exchange Administrative Group ...`.

The known attachment path reported by Tika is:

```text
/ First email.msg/First email.msg/attachment.docx
```

This is the next preferred PSTD fixture because it provides a real attachment and multiple messages without being too large for normal CI use.

### Apache Tika `testPST_variousBodyTypes.pst`

- Upstream repository: `apache/tika`
- Pinned commit: `63e22d08ef249cc73a6d02da7bc199fc3623a607`
- Upstream path: `tika-parsers/tika-parsers-standard/tika-parsers-standard-modules/tika-parser-microsoft-module/src/test/resources/test-documents/testPST_variousBodyTypes.pst`
- Source URL: `https://github.com/apache/tika/blob/63e22d08ef249cc73a6d02da7bc199fc3623a607/tika-parsers/tika-parsers-standard/tika-parsers-standard-modules/tika-parser-microsoft-module/src/test/resources/test-documents/testPST_variousBodyTypes.pst`
- Upstream project licence: Apache License 2.0

Apache Tika's regression test expects five recursive metadata objects and uses the fixture specifically to exercise PST messages with different body forms. PSTD should use it after basic attachment extraction to validate independent plain-text, HTML, and RTF selection rather than relying only on the current HTML-derived RTF case.

### java-libpst `dist-list.pst`

- Upstream repository: `rjohnsondev/java-libpst`
- Pinned commit: `f158a64acf2a0e46ac3bd699bc7a5a8da6c40d26`
- Upstream path: `src/test/resources/dist-list.pst`
- Source URL: `https://github.com/rjohnsondev/java-libpst/blob/f158a64acf2a0e46ac3bd699bc7a5a8da6c40d26/src/test/resources/dist-list.pst`
- Upstream repository publishes Apache 2.0 and LGPL licence texts. Preserve this provenance if the fixture is redistributed.

java-libpst's tests document:

- a recurring appointment object;
- three deleted occurrence dates;
- two modified recurrence exceptions;
- exception descriptions `This is the appointment at 9` and `This is the one at 10`;
- a distribution list with three members;
- one stored contact and two one-off recipients.

This fixture should be introduced only after ordinary mail-message extraction is stable across the Tika samples. Appointment and distribution-list objects must not be forced through the normal email path.

## Address-type boundary

The address in `tika-testPST.pst` beginning `/o=ExchangeLabs/...` is a legacy Exchange `EX` address or distinguished name. It is X.500-style evidence, not a true X.400 O/R address.

Use it to validate this bounded behaviour:

```text
legacy Exchange address
    -> preserve the raw address and address type
    -> search only validated same-PST evidence for an SMTP mapping
    -> emit SMTP when uniquely resolved
    -> otherwise retain the raw address and mark it unresolved
```

Do not label an Exchange legacy DN as X.400.

No compact public fixture with documented true X.400 recipient rows and exact expected values was identified. A true X.400 fixture should therefore be generated synthetically from controlled data. It should contain at least:

- one recipient with `PidTagAddressType = X400`;
- one known O/R address such as `C=AU;A= ;P=PSTD;O=Fixture;S=Recipient;G=X400`;
- one `EX` recipient with a known legacy DN;
- separate, uniquely attributable SMTP evidence for both resolvable cases;
- one deliberately unresolvable address to prove fail-closed behaviour.

## Required verification

Run this before using or changing the corpus:

```bash
cd tests/fixtures/upstream
sha256sum --check SHA256SUMS

for fixture in *.pst; do
  test "$(od -An -tx1 -N4 "$fixture" | tr -d ' \n')" = '2142444e'
done
```

The four-byte PST signature is `!BDN`, represented as hexadecimal `21 42 44 4e`.

Never replace a fixture in place. Add a new pinned file or intentionally update the upstream commit, checksum, size, documentation, and expected evidence in one reviewed change.

## Development order

### Stage 1: known DOCX attachment

Fixture: `tika-testPST.pst`

Preferred vertical milestones:

1. discover the message that owns the documented attachment row;
2. expose one validated attachment filename;
3. expose attachment size and method;
4. resolve the exact attachment payload bytes;
5. validate the DOCX signature and checksum;
6. emit the attachment through structured output;
7. assemble a deterministic `multipart/mixed` EML containing the existing body alternatives and the DOCX attachment.

Each step must report exact message, body, recipient, attachment, EML, and output-byte counts. Do not add attachment abstractions that do not expose one of these values from the fixture.

### Stage 2: multiple messages and folders

Fixture: `tika-testPST.pst`

Validate:

- folder hierarchy and non-ASCII folder names;
- all actual message candidates without false positives;
- stable message-to-folder attribution;
- Unicode subject, sender, and recipient values;
- one EML per eligible message;
- explicit completeness status for messages that cannot yet be emitted.

### Stage 3: body representation selection

Fixture: `tika-various-body-types.pst`

Validate plain text, independent HTML, RTF, and fallback ordering. Do not infer HTML from RTF when a separately stored validated HTML property is available.

### Stage 4: appointments and recurrence

Fixture: `java-libpst-dist-list.pst`

Preferred vertical milestones:

1. identify an appointment through `PidTagMessageClass`;
2. expose subject, start time, end time, location, and time-zone evidence;
3. decode recurrence metadata;
4. expose deleted occurrences;
5. expose modified exceptions and their descriptions;
6. generate ICS only after the source properties are independently validated.

### Stage 5: contacts and distribution lists

Fixture: `java-libpst-dist-list.pst`

Validate one stored contact and two one-off distribution-list entries. Preserve their source record types rather than flattening them prematurely into normal email recipients.

### Stage 6: true X.400

Use a controlled synthetic fixture after the public corpus has established legacy Exchange address handling. Keep raw X.400, EX, X.500-style, and SMTP representations distinct throughout extraction.

## CI policy

The original small public fixture remains the fast required regression gate for the already validated readable-email path. The upstream corpus should be added to CI incrementally as each corresponding vertical milestone becomes deterministic.

Do not make all three fixtures a blanket success gate before PSTD supports their object types. Instead, add narrowly scoped workflows such as:

- `tika-attachment-fixture`;
- `tika-multi-message-fixture`;
- `tika-body-types-fixture`;
- `java-libpst-calendar-fixture`.

Each workflow must name its exact fixture, expected values, and output counts. Avoid global `find ... | head -n 1` fixture selection once multiple PSTs are present.

## Data-safety rules

- Never commit private client PSTs.
- Use immutable upstream commit URLs, not moving branch URLs.
- Retain upstream repository, commit, path, licence, size, and SHA-256 evidence.
- Treat message content as test data and avoid publishing additional personal information beyond what upstream tests already document.
- Quarantine corrupt or adversarial PSTs separately from the well-formed fixture lane.
- Do not claim broad compatibility from passing this three-file corpus.

## Immediate next milestone

Run PSTD against `tests/fixtures/upstream/tika-testPST.pst` and extract the first validated attachment field from the known DOCX-bearing message. The preferred first observable result is the attachment filename. If filename resolution cannot be validated, expose the smallest real attachment property—such as size, method, or count—that can be tied unambiguously to that message.
