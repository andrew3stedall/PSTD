from pathlib import Path


def replace_one(path: str, old: str, new: str, label: str) -> None:
    file = Path(path)
    text = file.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: {label} expected one match, found {count}")
    file.write_text(text.replace(old, new))


replace_one("README.md", "_Last reviewed: 17 July 2026._", "_Last reviewed: 18 July 2026._", "review date")
replace_one(
    "README.md",
    """| Vertical extraction sequence | Complete through Vertical 34 / PR #455 | The Tika method-`5` object now emits as a separate linked message with its directly owned recipient and bodies, while the proven DOCX and outer EML remain unchanged. |
| Current milestone | Embedded child EML | Assemble a deterministic plain-text EML for the recovered child without treating its four-byte raw HTML evidence as markup. |
| EML reconstruction | Two fixture paths validated | The original fixture emits one deterministic 956-byte plain/HTML EML; the Tika fixture emits one deterministic 17,035-byte plain-text/DOCX `multipart/mixed` EML. |
""",
    """| Vertical extraction sequence | Complete through Vertical 35 / PR #457 | The Tika method-`5` child now emits as a deterministic attachmentless plain-text EML while ordinary unvalidated plain-only messages remain fail-closed. |
| Current milestone | Method-5 child attachment payload | Materialise the exact child EML as the method-`5` `message/rfc822` payload with deterministic bytes, hash, path, and ownership. |
| EML reconstruction | Three deterministic outputs across two fixtures | The original fixture emits one 956-byte plain/HTML EML; Tika emits the unchanged 17,035-byte plain-text/DOCX parent and one exact 453-byte plain-text child. |
""",
    "headline status",
)
replace_one(
    "README.md",
    "The Tika attachment fixture now emits eight top-level messages/recipients plus one separately linked embedded message and its directly owned raw/native recipient. Six top-level rows carry validated SMTP addresses; the other three recipient rows preserve native evidence without guessing SMTP. The attachment owner still emits the same deterministic 17,035-byte `multipart/mixed` EML and byte-identical DOCX. The child preserves a 23-byte text body and four raw HTML-property bytes without leaking them into its parent or treating them as usable MIME HTML.",
    "The Tika attachment fixture now emits seven top-level messages plus one separately linked embedded child, nine directly owned recipient records, ten body records, two attachment records, one exact DOCX payload, and two deterministic EML files. The attachment owner retains its 17,035-byte `multipart/mixed` EML and byte-identical DOCX. The child emits as a 453-byte single-part plain-text EML while its four raw HTML-property bytes remain structured evidence and are excluded from MIME output.",
    "Tika summary",
)
replace_one(
    "README.md",
    "| Vertical 34 / #455 | Parsed the exact PtypObject wrapper, resolved one unique normal-message NID, and emitted the linked child message, one recipient, and two raw body records without changing the outer EML. |",
    "| Vertical 34 / #455 | Parsed the exact PtypObject wrapper, resolved one unique normal-message NID, and emitted the linked child message, one recipient, and two raw body records without changing the outer EML. |\n| Vertical 35 / #457 | Emitted the linked child as an exact 453-byte attachmentless `text/plain` EML, authorised from attachment metadata and excluding unrelated unvalidated plain-only messages. |",
    "progress row",
)
replace_one(
    "README.md",
    "PSTD is not yet a general-purpose or absolute-coverage PST-to-EML converter. Current evidence is fixture-limited. The Tika sender remains a raw native Exchange distinguished name rather than resolved SMTP; embedded children are structured but not yet emitted as standalone/plain-text EML payloads; nested child attachments and uncommon/corrupt layouts remain incomplete. Do not infer broad compatibility from the milestone count.",
    "PSTD is not yet a general-purpose or absolute-coverage PST-to-EML converter. Current evidence is fixture-limited. The Tika sender remains a raw native Exchange distinguished name rather than resolved SMTP; the recovered child emits as a standalone EML but is not yet materialised as the method-`5` attachment payload; nested child attachments and uncommon/corrupt layouts remain incomplete. Do not infer broad compatibility from the milestone count.",
    "limitations",
)

replace_one("docs/product/project-status.md", "_Last reviewed: 17 July 2026._", "_Last reviewed: 18 July 2026._", "review date")
replace_one(
    "docs/product/project-status.md",
    "| Embedded message | Validated through Vertical 34 / PR #455 | One method-`5` PtypObject resolves to a separate linked message with one directly owned recipient, a 23-byte text body, and four preserved raw HTML-property bytes; no child evidence is projected onto the parent. |",
    "| Embedded message | Validated through Vertical 34 / PR #455 | One method-`5` PtypObject resolves to a separate linked message with one directly owned recipient, a 23-byte text body, and four preserved raw HTML-property bytes; no child evidence is projected onto the parent. |\n| Embedded child EML | Validated through Vertical 35 / PR #457 | The linked child emits one deterministic 453-byte single-part `text/plain` EML with exact headers, CRLF body assembly, and SHA-256; raw HTML bytes remain excluded, and unrelated plain-only messages remain unavailable. |",
    "child EML row",
)
replace_one(
    "docs/product/project-status.md",
    """## Tika Vertical 34 evidence

| Metric | Vertical 33 baseline | PR #455 result |
|---|---:|---:|
| Messages | 7 | 8 |
| Body records | 8 | 10 |
| Body payload files / bytes | 6 / 252 | 8 / 279 |
| Recipient records | 8 | 9 |
| Recipient JSONL bytes | 2,418 | 2,708 |
| Attachment records | 1 | 2 |
| Attachment payload files / bytes | 1 / 11,862 | 1 / 11,862 |
| EML files / bytes | 1 / 17,035 | 1 / 17,035 |
| Extraction TAR bytes | 202,752 | 227,840 |
| Total extraction-output bytes | 241,579 | 272,884 |

The method-`5` Property Context now preserves its object HNID, validates the exact eight-byte PtypObject `Nid + ulSize` allocation, requires a normal-message NID, and resolves that NID exactly once within the outer message's loaded subnode scope. The linked child `msg_0ff529af59d373d5` owns its own recipient and body records. The parent keeps its original recipient, DOCX ordinal/key/path, and 17,035-byte EML.

The child's four-byte `PidTagHtml` evidence is `7f 83 00 00`. It remains a raw body artefact and is not promoted to MIME HTML. The method-`5` attachment is metadata-only and links to the child with `embedded_message_key`; no empty EML payload is written at its archive path.
""",
    """## Tika Vertical 35 evidence

| Metric | Vertical 34 baseline | PR #457 result |
|---|---:|---:|
| Messages | 8 | 8 |
| Body records | 10 | 10 |
| Body payload files / bytes | 8 / 279 | 8 / 279 |
| Recipient records | 9 | 9 |
| Recipient JSONL bytes | 2,708 | 2,708 |
| Attachment records | 2 | 2 |
| Attachment payload files / bytes | 1 / 11,862 | 1 / 11,862 |
| EML files / bytes | 1 / 17,035 | 2 / 17,488 |
| Extraction TAR bytes | 227,840 | 227,840 |
| Total extraction-output bytes | 272,884 | 272,884 |

The linked child `msg_0ff529af59d373d5` now emits a deterministic 453-byte single-part `text/plain` EML with SHA-256 `86ffe5567da7aa505b8be16400889170ca583fd247cc0758f00a43c2a8a99420`. Its From, To, Subject, Date, Message-ID, MIME-Version, transfer encoding, CRLF line endings, and exact 23-byte body evidence are locked by the fixture workflow. The parent keeps its original recipient, DOCX ordinal/key/path, and 17,035-byte EML.

Plain-text-only admission is authorised from `AttachmentRecord.embedded_message_key`, not from payload availability. Ordinary top-level messages without validated HTML remain unavailable. The child's four-byte `PidTagHtml` evidence `7f 83 00 00` remains a raw body artefact and is absent from MIME output. The method-`5` attachment is still metadata-only and has no published payload file.
""",
    "Vertical evidence section",
)
replace_one(
    "docs/product/project-status.md",
    "PR #455, **Recover Tika embedded message as a separate object**, preserves PtypObject HNIDs before heap dereference, decodes the specification-defined object wrapper, requires one unambiguous child NID in the parent message scope, isolates the child's subnode subtree, and reuses the existing message/body/direct-recipient projections. Ambiguous, missing, malformed, wrong-type, or duplicate references remain unavailable.",
    "PR #457, **Emit recovered child as plain-text EML**, adds a policy-gated attachmentless EML path for message keys referenced by authoritative attachment metadata. It emits one exact child EML, retains the parent's MIME and DOCX bytes, and preserves fail-closed behaviour for unrelated plain-only records. Missing headers, invalid UTF-8, boundary collisions, ambiguous links, and unsupported candidates remain unavailable.",
    "latest completed",
)
replace_one(
    "docs/product/project-status.md",
    "Emit a deterministic plain-text-only EML for the recovered child. Its validated sender, recipient, subject, received-time Date evidence, Message-ID, and 23-byte UTF-8 body are available, but the current attachmentless EML path requires a validated HTML alternative. The raw four-byte HTML property must remain excluded. Materialising that child EML as the parent method-`5` attachment payload remains a later explicit boundary.",
    "Materialise the exact 453-byte child EML as the method-`5` attachment payload. The new payload must use `message/rfc822`, retain explicit parent-child ownership, publish deterministic path/length/SHA-256 evidence, and replace the current metadata-only boundary without writing an empty placeholder. Nested recursion and outer-parent MIME inclusion remain separate unless independently validated in the same fixture slice.",
    "next milestone",
)

replace_one("docs/product/pstd-v1-roadmap.md", "_Last reviewed: 17 July 2026._", "_Last reviewed: 18 July 2026._", "review date")
replace_one(
    "docs/product/pstd-v1-roadmap.md",
    "The child owns one raw/native recipient, a 23-byte UTF-8 text body, and four raw `PidTagHtml` bytes. Its subtree is isolated before recipient projection, so none of those values enter the parent. The outer DOCX remains ordinal `0`, and its 17,035-byte EML is unchanged. Exact evidence is recorded in [Vertical 34](../operations/vertical-34-recover-tika-embedded-message.md).",
    "The child owns one raw/native recipient, a 23-byte UTF-8 text body, and four raw `PidTagHtml` bytes. Its subtree is isolated before recipient projection, so none of those values enter the parent. The outer DOCX remains ordinal `0`, and its 17,035-byte EML is unchanged. Exact evidence is recorded in [Vertical 34](../operations/vertical-34-recover-tika-embedded-message.md).\n\n### Recovered child plain-text EML\n\nComplete in PR #457. The linked child now emits one deterministic 453-byte attachmentless `text/plain` EML. Admission is restricted to message keys referenced by authoritative attachment metadata, so unrelated top-level plain-only records remain fail-closed. The child preserves native Exchange addresses, validated headers, exact CRLF body assembly, and SHA-256 `86ffe5567da7aa505b8be16400889170ca583fd247cc0758f00a43c2a8a99420`; its raw `7f 83 00 00` HTML evidence is excluded. Exact evidence is recorded in [Vertical 35](../operations/vertical-35-emit-tika-child-eml.md).",
    "completed child EML section",
)
replace_one(
    "docs/product/pstd-v1-roadmap.md",
    """### Emit the recovered child as a plain-text EML

The recovered child already has validated sender, recipient, subject, received-time Date evidence, Message-ID, and UTF-8 plain text. Its four HTML-property bytes are not usable markup. The next smallest vertical must:

- permit a deterministic attachmentless `text/plain` EML when all required headers and plain body validate but no HTML alternative exists;
- emit exactly one new child EML without changing the parent's 17,035-byte multipart EML;
- prove that `7f 83 00 00` is absent from MIME output;
- retain the raw/native Exchange addresses without inventing SMTP;
- keep method-`5` TAR payload materialisation separate unless the generated child EML is explicitly adopted as that payload.
""",
    """### Materialise the method-5 child attachment payload

The exact 453-byte child EML is now available. The next smallest vertical must:

- adopt those exact bytes as the method-`5` attachment payload rather than writing an empty placeholder;
- publish content type `message/rfc822`, deterministic archive path, byte length, and SHA-256;
- preserve the existing attachment key, ordinal, parent message key, and `embedded_message_key` relationship;
- reject missing, duplicate, mismatched, or non-message child links;
- keep nested recursion and broader method-`5` layouts out of scope;
- include the new payload in the outer parent EML only if that MIME change is separately asserted and proven without changing the DOCX bytes.
""",
    "current milestone",
)
replace_one("docs/product/pstd-v1-roadmap.md", "After child EML assembly:", "After method-`5` child payload materialisation:", "following sequence")

replace_one(
    "docs/product/compatibility-matrix.md",
    "| Transport or FILETIME Date | Exact | Exact for emitted parent; child evidence available | Child EML and conflicting-Date fixtures |",
    "| Transport or FILETIME Date | Exact | Exact for emitted parent and child | Conflicting-Date and missing-Date fixtures |",
    "date coverage",
)
replace_one(
    "docs/product/compatibility-matrix.md",
    "| Plain-text-only attachmentless EML | Not exercised | Partial: child has complete evidence but current EML path requires HTML | Emit deterministic child EML |",
    "| Plain-text-only attachmentless EML | Not exercised | Exact: 453-byte child EML, gated to an attachment-linked embedded message | Additional independently validated plain-only messages |",
    "plain EML coverage",
)
replace_one(
    "docs/product/compatibility-matrix.md",
    "| Method-5 embedded-message link | Not exercised | Exact: one separately owned child | Materialised child EML payload |",
    "| Method-5 embedded-message link | Not exercised | Exact: one separately owned child with exact standalone EML | Materialised `message/rfc822` attachment payload |",
    "method5 link",
)
replace_one(
    "docs/product/compatibility-matrix.md",
    "| Embedded-message attachment payload | Not exercised | Partial: link exists; no payload published | Deterministic `message/rfc822` payload |",
    "| Embedded-message attachment payload | Not exercised | Partial: exact child EML exists; attachment payload is still metadata-only | Deterministic `message/rfc822` payload |",
    "embedded payload",
)
replace_one(
    "docs/product/compatibility-matrix.md",
    "| Deterministic output | Exact | Exact for current locked outputs | Whole-corpus repeatability checks |",
    "| Deterministic output | Exact | Exact for the 17,035-byte parent and 453-byte child EMLs | Whole-corpus repeatability checks |",
    "determinism",
)
replace_one(
    "docs/product/compatibility-matrix.md",
    "PSTD currently demonstrates a material Unicode email extraction path, including recipients, bodies, one by-value DOCX attachment, one attachment-bearing EML, and one separately recovered embedded message. This is evidence for the approved fixtures only. It is not evidence of general PST compatibility.",
    "PSTD currently demonstrates a material Unicode email extraction path, including recipients, bodies, one by-value DOCX attachment, one attachment-bearing parent EML, and one separately recovered child with an exact standalone plain-text EML. This is evidence for the approved fixtures only. It is not evidence of general PST compatibility.",
    "release interpretation",
)
replace_one(
    "docs/product/compatibility-matrix.md",
    """1. Emit the recovered Tika child as deterministic plain-text-only EML while excluding the invalid four-byte HTML property.
2. Materialise that EML as the method-5 attachment payload with `message/rfc822`, exact bytes, hash, path, and parent-child ownership checks.
3. Lock complete folder and message coverage for the Tika fixture.
4. Add independent valid plain-text, HTML, and RTF body fixtures.
5. Add the first pinned public ANSI PST and establish header, tree, folder, message, recipient, and body baselines before extending deeper features.
""",
    """1. Materialise the exact child EML as the method-5 attachment payload with `message/rfc822`, exact bytes, hash, path, and parent-child ownership checks.
2. Lock complete folder and message coverage for the Tika fixture.
3. Add independent valid plain-text, HTML, and RTF body fixtures.
4. Add the first pinned public ANSI PST and establish header, tree, folder, message, recipient, and body baselines before extending deeper features.
5. Broaden attachment methods, inline/CID handling, nested messages, and authoritative Exchange-to-SMTP resolution from additional fixtures.
""",
    "immediate sequence",
)

replace_one("docs/operations/upstream-pst-fixture-corpus.md", "_Last reviewed: 16 July 2026._", "_Last reviewed: 18 July 2026._", "review date")
replace_one(
    "docs/operations/upstream-pst-fixture-corpus.md",
    "PSTD currently discovers seven normal message candidates and eight body records. The known DOCX-bearing outer message is:",
    "PSTD currently emits seven top-level messages plus one separately linked method-`5` child, ten body records, nine directly owned recipient records, and two attachment records. The known DOCX-bearing outer message is:",
    "current Tika counts",
)
replace_one(
    "docs/operations/upstream-pst-fixture-corpus.md",
    "Filename evidence is recorded in [Vertical 29](vertical-29-expose-docx-attachment-filename.md), exact reference-resolution evidence in [Vertical 30](vertical-30-resolve-docx-attachment-data-reference.md), and payload evidence in [Vertical 31](vertical-31-emit-docx-attachment-payload.md).\n\nThe immediate next use of this fixture is recipient extraction for the same message. It currently emits zero recipient records, so EML assembly remains deferred even though subject, sender, identifiers, plain text, HTML, and the DOCX attachment are available.",
    "Filename evidence is recorded in [Vertical 29](vertical-29-expose-docx-attachment-filename.md), exact reference-resolution evidence in [Vertical 30](vertical-30-resolve-docx-attachment-data-reference.md), and payload evidence in [Vertical 31](vertical-31-emit-docx-attachment-payload.md). Recipient and outer-EML evidence is recorded in Verticals 32-33, embedded-message recovery in [Vertical 34](vertical-34-recover-tika-embedded-message.md), and exact child EML evidence in [Vertical 35](vertical-35-emit-tika-child-eml.md).\n\nThe fixture now produces two deterministic EML files: the unchanged 17,035-byte parent with the exact DOCX and a 453-byte single-part plain-text child. The child EML is authorised through the method-`5` attachment metadata link and excludes its four raw non-markup HTML bytes. The next use of this fixture is to materialise those exact child bytes as the `message/rfc822` attachment payload, followed by complete folder/message coverage validation.",
    "stale next use",
)

replace_one("docs/README.md", "_Last reviewed: 17 July 2026._", "_Last reviewed: 18 July 2026._", "review date")
replace_one(
    "docs/README.md",
    "| Tika attachment fixture | Eight messages now include one separately linked method-`5` child, nine directly owned recipients, ten body records, the unchanged DOCX payload, and the unchanged outer multipart EML. |\n| Current milestone | Emit a deterministic plain-text-only EML for the recovered child while excluding its four raw non-markup HTML bytes. |",
    "| Tika attachment fixture | Eight messages include one linked method-`5` child, nine directly owned recipients, ten body records, the unchanged DOCX payload, the unchanged 17,035-byte parent EML, and one exact 453-byte child EML. |\n| Current milestone | Materialise the exact child EML as the method-`5` `message/rfc822` attachment payload. |",
    "current extraction state",
)
replace_one(
    "docs/README.md",
    "- [Vertical 34: Tika embedded message](operations/vertical-34-recover-tika-embedded-message.md)",
    "- [Vertical 34: Tika embedded message](operations/vertical-34-recover-tika-embedded-message.md)\n- [Vertical 35: Tika child plain-text EML](operations/vertical-35-emit-tika-child-eml.md)",
    "vertical link",
)

replace_one("docs/changelog/unreleased.md", "_Last reviewed: 17 July 2026._", "_Last reviewed: 18 July 2026._", "review date")
replace_one(
    "docs/changelog/unreleased.md",
    "- A permanent Vertical 34 fixture contract covering exact child/parent ownership, stable attachment ordinals, record bytes, archive bytes, and unchanged outer EML.",
    "- A permanent Vertical 34 fixture contract covering exact child/parent ownership, stable attachment ordinals, record bytes, archive bytes, and unchanged outer EML.\n- One deterministic 453-byte attachmentless `text/plain` EML for the linked method-`5` child, with exact SHA-256, headers, CRLF body, and exclusion of raw HTML bytes.\n- Policy-gated plain-text-only EML admission from authoritative attachment metadata, retaining fail-closed behaviour for unrelated unvalidated plain-only messages.",
    "added bullets",
)
replace_one("docs/changelog/unreleased.md", "EML files/bytes: 1/17035", "EML files/bytes: 2/17488", "Tika metrics")
replace_one(
    "docs/changelog/unreleased.md",
    "- Emit a deterministic attachmentless plain-text EML for the recovered child while excluding its four non-markup HTML bytes and preserving raw/native address evidence.",
    "- Materialise the exact 453-byte child EML as the method-`5` `message/rfc822` attachment payload with deterministic path, hash, and ownership.",
    "in progress",
)
replace_one(
    "docs/changelog/unreleased.md",
    "- The Tika fixture has one validated outer EML; the recovered attachmentless child does not yet emit a plain-text-only EML.",
    "- The Tika fixture has exact parent and child EMLs, but the child bytes are not yet published as the method-`5` attachment payload.",
    "known limitation",
)

replace_one("docs/operations/public-pst-progress-log.md", "_Last reviewed: 17 July 2026._", "_Last reviewed: 18 July 2026._", "review date")
replace_one(
    "docs/operations/public-pst-progress-log.md",
    "## Tika recipient, attachment, and embedded-message evidence\n\nPR #455 advances `tests/fixtures/upstream/tika-testPST.pst` from seven top-level messages to eight structured messages by recovering the method-`5` child separately.",
    "## Tika recipient, attachment, embedded-message, and child-EML evidence\n\nPR #457 retains the eight-message structured baseline and adds one deterministic standalone EML for the linked method-`5` child.",
    "Tika heading",
)
replace_one("docs/operations/public-pst-progress-log.md", "| EML files / bytes | 1 / 17,035 |", "| EML files / bytes | 2 / 17,488 |", "EML evidence")
replace_one(
    "docs/operations/public-pst-progress-log.md",
    "The original seven messages and eight recipients remain byte-for-byte stable. The recovered child adds one raw/native recipient, a 23-byte text body, and four raw HTML-property bytes. The method-`5` parent attachment links to the child key but remains metadata-only; the existing DOCX retains ordinal `0`, key `att_0695091e19397627`, exact bytes, and MIME EML. No child value is attributed to the parent.",
    "The original seven top-level messages and eight top-level recipients remain stable. The recovered child owns one raw/native recipient, a 23-byte text body, four raw HTML-property bytes, and one exact 453-byte single-part plain-text EML. The method-`5` parent attachment links to the child key but remains metadata-only; the existing DOCX retains ordinal `0`, key `att_0695091e19397627`, exact bytes, and its unchanged 17,035-byte parent EML. No child value is attributed to the parent.",
    "Tika evidence paragraph",
)
replace_one(
    "docs/operations/public-pst-progress-log.md",
    "| 2026-07-17 | Vertical 34 / #455 | Material embedded-message extraction | Parsed the exact PtypObject wrapper, resolved one unique normal-message NID, emitted a separately keyed child with one recipient and two body records, linked it from method `5`, and preserved the parent/DOCX/EML contract. | Emit a plain-text-only child EML without promoting four invalid HTML bytes. |",
    "| 2026-07-18 | Vertical 35 / #457 | Material child EML assembly | Emitted one exact 453-byte single-part plain-text child EML, gated admission through attachment metadata, preserved fail-closed top-level behaviour, and retained the parent/DOCX bytes. | Materialise the exact child EML as the method-`5` `message/rfc822` attachment payload. |\n| 2026-07-17 | Vertical 34 / #455 | Material embedded-message extraction | Parsed the exact PtypObject wrapper, resolved one unique normal-message NID, emitted a separately keyed child with one recipient and two body records, linked it from method `5`, and preserved the parent/DOCX/EML contract. | Emit a plain-text-only child EML without promoting four invalid HTML bytes. |",
    "history row",
)
replace_one(
    "docs/operations/public-pst-progress-log.md",
    "Vertical 34 completes separate structured recovery of the Tika method-`5` child. The next slice is a deterministic plain-text-only child EML; the current attachmentless EML assembler still requires validated HTML.",
    "Vertical 35 completes deterministic standalone EML assembly for the Tika method-`5` child. The next slice is exact `message/rfc822` attachment payload materialisation with path, hash, ownership, and fail-closed link validation.",
    "active boundary",
)
replace_one(
    "docs/operations/public-pst-progress-log.md",
    "The parser has advanced from structural discovery to material recipient, body, attachment, and readable-EML output on approved fixtures. This is still not broad compatibility: the Tika sender remains a raw native Exchange DN rather than resolved SMTP, nested embedded attachments and child-EML materialisation remain deferred, and one fixture cannot establish support for uncommon or corrupt layouts.",
    "The parser has advanced from structural discovery to material recipient, body, attachment, parent-EML, and child-EML output on approved fixtures. This is still not broad compatibility: the Tika sender remains a raw native Exchange DN rather than resolved SMTP, method-`5` attachment payloads and nested embedded attachments remain incomplete, and one fixture cannot establish support for uncommon or corrupt layouts.",
    "interpretation",
)

replace_one("docs/DOCUMENTATION_STATUS.md", "_Last reviewed: 14 July 2026._", "_Last reviewed: 18 July 2026._", "review date")
replace_one(
    "docs/DOCUMENTATION_STATUS.md",
    """As of this review:

- merged implementation is complete through PR #429 / Vertical 13;
- draft PR #430 is active and unmerged;
- complete recipient record assembly is validated on `main`;
- same-run projection and production publication remain incomplete;
- downstream Snowflake/UI/search work remains parked;
""",
    """As of this review:

- merged implementation is complete through PR #457 / Vertical 35;
- the Tika fixture emits eight structured messages, nine recipients, one exact DOCX payload, one 17,035-byte parent EML, and one 453-byte child EML;
- the active boundary is method-`5` `message/rfc822` attachment payload materialisation;
- fixture breadth, ANSI support, nested messages, non-mail object coverage, and broad address/body fidelity remain incomplete;
- downstream Snowflake/UI/search work remains parked;
""",
    "documentation baseline",
)
