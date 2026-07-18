from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    text = target.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one replacement, found {count}: {old[:100]!r}")
    target.write_text(text.replace(old, new, 1))


replace_once(
    "README.md",
    "| Vertical extraction sequence | Complete through Vertical 35 / PR #457 | The Tika method-`5` child now emits as a deterministic attachmentless plain-text EML while ordinary unvalidated plain-only messages remain fail-closed. |\n| Current milestone | Method-5 child attachment payload | Materialise the exact child EML as the method-`5` `message/rfc822` payload with deterministic bytes, hash, path, and ownership. |",
    "| Vertical extraction sequence | Complete through Vertical 36 / PR #461 | The exact 453-byte child EML is now also the authoritative method-`5` `message/rfc822` attachment payload, with unique-link and nested-child rejection. |\n| Current milestone | Complete Tika folder/message validation | Lock exact folder paths, message ownership, Unicode names, and legacy Exchange address preservation across the full Tika fixture. |",
)
replace_once(
    "README.md",
    "The Tika attachment fixture now emits seven top-level messages plus one separately linked embedded child, nine directly owned recipient records, ten body records, two attachment records, one exact DOCX payload, and two deterministic EML files. The attachment owner retains its 17,035-byte `multipart/mixed` EML and byte-identical DOCX. The child emits as a 453-byte single-part plain-text EML while its four raw HTML-property bytes remain structured evidence and are excluded from MIME output.",
    "The Tika attachment fixture now emits seven top-level messages plus one separately linked embedded child, nine directly owned recipient records, ten body records, two attachment records, two exact attachment payloads totalling 12,315 bytes, and two deterministic EML files. The method-`5` payload is byte-identical to the 453-byte standalone child EML and uses `message/rfc822`. The attachment owner retains its 17,035-byte `multipart/mixed` EML and byte-identical DOCX; method `5` is deliberately not inserted into the parent MIME tree.",
)
replace_once(
    "README.md",
    "| Vertical 35 / #457 | Emitted the linked child as an exact 453-byte attachmentless `text/plain` EML, authorised from attachment metadata and excluding unrelated unvalidated plain-only messages. |",
    "| Vertical 35 / #457 | Emitted the linked child as an exact 453-byte attachmentless `text/plain` EML, authorised from attachment metadata and excluding unrelated unvalidated plain-only messages. |\n| Vertical 36 / #461 | Materialised those exact bytes as the method-`5` `message/rfc822` payload, preserved the parent EML/DOCX, and added fail-closed link and nesting tests. |",
)
replace_once(
    "README.md",
    "PSTD is not yet a general-purpose or absolute-coverage PST-to-EML converter. Current evidence is fixture-limited. The Tika sender remains a raw native Exchange distinguished name rather than resolved SMTP; the recovered child emits as a standalone EML but is not yet materialised as the method-`5` attachment payload; nested child attachments and uncommon/corrupt layouts remain incomplete. Do not infer broad compatibility from the milestone count.",
    "PSTD is not yet a general-purpose or absolute-coverage PST-to-EML converter. Current evidence is fixture-limited. The Tika sender remains a raw native Exchange distinguished name rather than resolved SMTP; one method-`5` layout is exact, but nested child attachments, broader producers, ANSI files, and uncommon/corrupt layouts remain incomplete. Do not infer broad compatibility from the milestone count.",
)

replace_once(
    "docs/product/project-status.md",
    "| Embedded child EML | Validated through Vertical 35 / PR #457 | The linked child emits one deterministic 453-byte single-part `text/plain` EML with exact headers, CRLF body assembly, and SHA-256; raw HTML bytes remain excluded, and unrelated plain-only messages remain unavailable. |",
    "| Embedded child EML | Validated through Vertical 35 / PR #457 | The linked child emits one deterministic 453-byte single-part `text/plain` EML with exact headers, CRLF body assembly, and SHA-256; raw HTML bytes remain excluded, and unrelated plain-only messages remain unavailable. |\n| Method-5 child payload | Validated through Vertical 36 / PR #461 | The same exact 453 bytes now publish at the existing method-`5` archive path as `message/rfc822`; key, ordinal, owner and child link remain stable, with fail-closed duplicate, mismatch and nesting rejection. |",
)
replace_once(
    "docs/product/project-status.md",
    "## Tika Vertical 35 evidence\n\n| Metric | Vertical 34 baseline | PR #457 result |\n|---|---:|---:|\n| Messages | 8 | 8 |\n| Body records | 10 | 10 |\n| Body payload files / bytes | 8 / 279 | 8 / 279 |\n| Recipient records | 9 | 9 |\n| Recipient JSONL bytes | 2,708 | 2,708 |\n| Attachment records | 2 | 2 |\n| Attachment payload files / bytes | 1 / 11,862 | 1 / 11,862 |\n| EML files / bytes | 1 / 17,035 | 2 / 17,488 |\n| Extraction TAR bytes | 227,840 | 227,840 |\n| Total extraction-output bytes | 272,884 | 272,884 |\n\nThe linked child `msg_0ff529af59d373d5` now emits a deterministic 453-byte single-part `text/plain` EML with SHA-256 `86ffe5567da7aa505b8be16400889170ca583fd247cc0758f00a43c2a8a99420`. Its From, To, Subject, Date, Message-ID, MIME-Version, transfer encoding, CRLF line endings, and exact 23-byte body evidence are locked by the fixture workflow. The parent keeps its original recipient, DOCX ordinal/key/path, and 17,035-byte EML.\n\nPlain-text-only admission is authorised from `AttachmentRecord.embedded_message_key`, not from payload availability. Ordinary top-level messages without validated HTML remain unavailable. The child's four-byte `PidTagHtml` evidence `7f 83 00 00` remains a raw body artefact and is absent from MIME output. The method-`5` attachment is still metadata-only and has no published payload file.",
    "## Tika Vertical 36 evidence\n\n| Metric | Vertical 35 baseline | PR #461 result |\n|---|---:|---:|\n| Messages | 8 | 8 |\n| Body records | 10 | 10 |\n| Body payload files / bytes | 8 / 279 | 8 / 279 |\n| Recipient records | 9 | 9 |\n| Attachment records | 2 | 2 |\n| Attachment payload files / bytes | 1 / 11,862 | 2 / 12,315 |\n| Attachment JSONL bytes | 1,358 | 1,240 |\n| EML files / bytes | 2 / 17,488 | 2 / 17,488 |\n| Extraction TAR bytes | 227,840 | 228,864 |\n| Total extraction-output bytes | 272,884 | 273,908 |\n\nThe method-`5` record `att_a9c94a13d70f1cb3` now publishes a 453-byte `message/rfc822` payload with SHA-256 `86ffe5567da7aa505b8be16400889170ca583fd247cc0758f00a43c2a8a99420` at its existing archive path. Those bytes are identical to standalone child `msg_0ff529af59d373d5.eml`. The parent message key, attachment ordinal, filename, archive path and `embedded_message_key` remain stable.\n\nThe shared EML builder is used by both output paths. Missing or mismatched links, duplicate child references, nested child ownership, ambiguous text bodies, unsafe headers and invalid EML evidence remain unavailable. Parent MIME assembly continues to admit only supported method-`1` by-value payloads, preserving the exact 17,035-byte parent EML and 11,862-byte DOCX.",
)
replace_once(
    "docs/product/project-status.md",
    "PR #457, **Emit recovered child as plain-text EML**, adds a policy-gated attachmentless EML path for message keys referenced by authoritative attachment metadata. It emits one exact child EML, retains the parent's MIME and DOCX bytes, and preserves fail-closed behaviour for unrelated plain-only records. Missing headers, invalid UTF-8, boundary collisions, ambiguous links, and unsupported candidates remain unavailable.",
    "PR #461, **Materialise method-5 child EML payload**, extracts deterministic plain-text EML assembly into a shared module and publishes the exact recovered-child bytes through the authoritative method-`5` attachment record. It retains all stable ownership identifiers and parent outputs while adding focused fail-closed tests and exact fixture evidence.",
)
replace_once(
    "docs/product/project-status.md",
    "Materialise the exact 453-byte child EML as the method-`5` attachment payload. The new payload must use `message/rfc822`, retain explicit parent-child ownership, publish deterministic path/length/SHA-256 evidence, and replace the current metadata-only boundary without writing an empty placeholder. Nested recursion and outer-parent MIME inclusion remain separate unless independently validated in the same fixture slice.",
    "Lock complete folder and message coverage for `tika-testPST.pst`: exact folder paths and counts, message-to-folder ownership, Unicode names, all eight structured messages, and preserved legacy Exchange/native address evidence. This should establish the fixture as a complete multi-message baseline before independent body-form selection work.",
)

replace_once(
    "docs/product/pstd-v1-roadmap.md",
    "## Current milestone\n\n### Materialise the method-5 child attachment payload\n\nThe exact 453-byte child EML is now available. The next smallest vertical must:\n\n- adopt those exact bytes as the method-`5` attachment payload rather than writing an empty placeholder;\n- publish content type `message/rfc822`, deterministic archive path, byte length, and SHA-256;\n- preserve the existing attachment key, ordinal, parent message key, and `embedded_message_key` relationship;\n- reject missing, duplicate, mismatched, or non-message child links;\n- keep nested recursion and broader method-`5` layouts out of scope;\n- include the new payload in the outer parent EML only if that MIME change is separately asserted and proven without changing the DOCX bytes.\n\n## Following fixture sequence\n\nAfter method-`5` child payload materialisation:\n\n1. validate multiple messages, folders, Unicode names, and legacy Exchange address preservation on `tika-testPST.pst`;\n2. validate body-form selection with `tika-various-body-types.pst`;\n3. validate appointments and recurrence exceptions with `java-libpst-dist-list.pst`;\n4. validate contacts and distribution-list entries without forcing them through the normal email path;\n5. create a controlled synthetic fixture for true X.400, because the public Exchange legacy DN is X.500-style/`EX`, not a true X.400 O/R address.",
    "### Method-5 child attachment payload\n\nComplete in PR #461. The exact 453-byte child EML is now published as the existing method-`5` attachment's `message/rfc822` payload. It retains the parent key, attachment key, ordinal, filename, archive path and `embedded_message_key`, and is byte-identical to the standalone child EML. Missing, duplicate, mismatched, nested and unsafe candidates remain fail-closed. Exact evidence is recorded in [Vertical 36](../operations/vertical-36-materialise-method5-eml-payload.md).\n\n## Current milestone\n\n### Lock complete Tika folder and message coverage\n\nThe next smallest vertical must:\n\n- assert all folder paths, names, parent relationships and counts in `tika-testPST.pst`;\n- attribute all seven top-level messages and the recovered child to the correct owners without false positives;\n- lock Unicode names and subjects where present;\n- preserve all nine recipient records, including raw/native legacy Exchange evidence;\n- retain the exact DOCX, parent EML, child EML and method-`5` payload contracts;\n- classify any non-mail or unsupported object explicitly rather than forcing it into EML.\n\n## Following fixture sequence\n\nAfter complete Tika folder/message validation:\n\n1. validate body-form selection with `tika-various-body-types.pst`;\n2. add the first pinned public ANSI PST baseline;\n3. validate appointments and recurrence exceptions with `java-libpst-dist-list.pst`;\n4. validate contacts and distribution-list entries without forcing them through the normal email path;\n5. create a controlled synthetic fixture for true X.400, because the public Exchange legacy DN is X.500-style/`EX`, not a true X.400 O/R address.",
)

replace_once(
    "docs/product/compatibility-matrix.md",
    "| Method-5 embedded-message link | Not exercised | Exact: one separately owned child with exact standalone EML | Materialised `message/rfc822` attachment payload |\n| Embedded-message attachment payload | Not exercised | Partial: exact child EML exists; attachment payload is still metadata-only | Deterministic `message/rfc822` payload |",
    "| Method-5 embedded-message link | Not exercised | Exact: one separately owned child with exact standalone EML and payload | Additional producer/layout evidence |\n| Embedded-message attachment payload | Not exercised | Exact: 453-byte `message/rfc822`, byte-identical to standalone child EML | Additional layouts and bounded recursion |",
)
replace_once(
    "docs/product/compatibility-matrix.md",
    "PSTD currently demonstrates a material Unicode email extraction path, including recipients, bodies, one by-value DOCX attachment, one attachment-bearing parent EML, and one separately recovered child with an exact standalone plain-text EML. This is evidence for the approved fixtures only. It is not evidence of general PST compatibility.",
    "PSTD currently demonstrates a material Unicode email extraction path, including recipients, bodies, one by-value DOCX attachment, one attachment-bearing parent EML, and one separately recovered child whose exact EML is also published as a method-`5` `message/rfc822` payload. This is evidence for the approved fixtures only. It is not evidence of general PST compatibility.",
)
replace_once(
    "docs/product/compatibility-matrix.md",
    "1. Materialise the exact child EML as the method-5 attachment payload with `message/rfc822`, exact bytes, hash, path, and parent-child ownership checks.\n2. Lock complete folder and message coverage for the Tika fixture.\n3. Add independent valid plain-text, HTML, and RTF body fixtures.\n4. Add the first pinned public ANSI PST and establish header, tree, folder, message, recipient, and body baselines before extending deeper features.\n5. Broaden attachment methods, inline/CID handling, nested messages, and authoritative Exchange-to-SMTP resolution from additional fixtures.",
    "1. Lock complete folder and message coverage for the Tika fixture.\n2. Add independent valid plain-text, HTML, and RTF body fixtures.\n3. Add the first pinned public ANSI PST and establish header, tree, folder, message, recipient, and body baselines before extending deeper features.\n4. Broaden attachment methods, inline/CID handling, nested messages, and authoritative Exchange-to-SMTP resolution from additional fixtures.\n5. Add deterministic corrupt and ambiguous fixture cases before production-readiness claims.",
)

replace_once(
    "docs/operations/public-pst-progress-log.md",
    "PR #457 retains the eight-message structured baseline and adds one deterministic standalone EML for the linked method-`5` child.",
    "PR #461 retains the eight-message and two-EML baseline and publishes the exact child EML as the linked method-`5` attachment payload.",
)
replace_once(
    "docs/operations/public-pst-progress-log.md",
    "| Attachment payload files / bytes | 1 / 11,862 |\n| EML files / bytes | 2 / 17,488 |\n| Recipient JSONL bytes | 2,708 |\n| Extraction TAR bytes | 227,840 |\n| Total output bytes | 272,884 |",
    "| Attachment payload files / bytes | 2 / 12,315 |\n| EML files / bytes | 2 / 17,488 |\n| Recipient JSONL bytes | 2,708 |\n| Extraction TAR bytes | 228,864 |\n| Total output bytes | 273,908 |",
)
replace_once(
    "docs/operations/public-pst-progress-log.md",
    "The original seven top-level messages and eight top-level recipients remain stable. The recovered child owns one raw/native recipient, a 23-byte text body, four raw HTML-property bytes, and one exact 453-byte single-part plain-text EML. The method-`5` parent attachment links to the child key but remains metadata-only; the existing DOCX retains ordinal `0`, key `att_0695091e19397627`, exact bytes, and its unchanged 17,035-byte parent EML. No child value is attributed to the parent.",
    "The original seven top-level messages and eight top-level recipients remain stable. The recovered child owns one raw/native recipient, a 23-byte text body, four raw HTML-property bytes, and one exact 453-byte single-part plain-text EML. The method-`5` record now publishes those exact bytes as `message/rfc822` at its stable archive path; the existing DOCX and unchanged 17,035-byte parent EML remain separate and byte-identical. No child value is attributed to the parent.",
)
replace_once(
    "docs/operations/public-pst-progress-log.md",
    "| 2026-07-18 | Vertical 35 / #457 | Material child EML assembly | Emitted one exact 453-byte single-part plain-text child EML, gated admission through attachment metadata, preserved fail-closed top-level behaviour, and retained the parent/DOCX bytes. | Materialise the exact child EML as the method-`5` `message/rfc822` attachment payload. |",
    "| 2026-07-18 | Vertical 36 / #461 | Material embedded-message payload | Published the exact 453-byte child EML as `message/rfc822`, locked path/hash/ownership and byte identity, rejected ambiguous and nested candidates, and preserved parent EML/DOCX bytes. | Lock complete Tika folder and message coverage. |\n| 2026-07-18 | Vertical 35 / #457 | Material child EML assembly | Emitted one exact 453-byte single-part plain-text child EML, gated admission through attachment metadata, preserved fail-closed top-level behaviour, and retained the parent/DOCX bytes. | Materialise the exact child EML as the method-`5` `message/rfc822` attachment payload. |",
)
replace_once(
    "docs/operations/public-pst-progress-log.md",
    "Vertical 35 completes deterministic standalone EML assembly for the Tika method-`5` child. The next slice is exact `message/rfc822` attachment payload materialisation with path, hash, ownership, and fail-closed link validation.",
    "Vertical 36 completes exact method-`5` `message/rfc822` payload materialisation. The next slice is complete Tika folder and message validation with exact paths, ownership, Unicode names and legacy Exchange evidence.",
)
replace_once(
    "docs/operations/public-pst-progress-log.md",
    "The parser has advanced from structural discovery to material recipient, body, attachment, parent-EML, and child-EML output on approved fixtures. This is still not broad compatibility: the Tika sender remains a raw native Exchange DN rather than resolved SMTP, method-`5` attachment payloads and nested embedded attachments remain incomplete, and one fixture cannot establish support for uncommon or corrupt layouts.",
    "The parser has advanced from structural discovery to material recipient, body, by-value attachment, parent/child EML, and exact method-`5` payload output on approved fixtures. This is still not broad compatibility: the Tika sender remains a raw native Exchange DN rather than resolved SMTP, nested embedded attachments and additional method-`5` layouts remain incomplete, and one fixture cannot establish support for uncommon or corrupt layouts.",
)

replace_once(
    "docs/operations/upstream-pst-fixture-corpus.md",
    "payload files/bytes: 1/11862",
    "payload files/bytes: 2/12315",
)
replace_once(
    "docs/operations/upstream-pst-fixture-corpus.md",
    "Filename evidence is recorded in [Vertical 29](vertical-29-expose-docx-attachment-filename.md), exact reference-resolution evidence in [Vertical 30](vertical-30-resolve-docx-attachment-data-reference.md), and payload evidence in [Vertical 31](vertical-31-emit-docx-attachment-payload.md). Recipient and outer-EML evidence is recorded in Verticals 32-33, embedded-message recovery in [Vertical 34](vertical-34-recover-tika-embedded-message.md), and exact child EML evidence in [Vertical 35](vertical-35-emit-tika-child-eml.md).",
    "Filename evidence is recorded in [Vertical 29](vertical-29-expose-docx-attachment-filename.md), exact reference-resolution evidence in [Vertical 30](vertical-30-resolve-docx-attachment-data-reference.md), and DOCX payload evidence in [Vertical 31](vertical-31-emit-docx-attachment-payload.md). Recipient and outer-EML evidence is recorded in Verticals 32-33, embedded-message recovery in [Vertical 34](vertical-34-recover-tika-embedded-message.md), exact child EML evidence in [Vertical 35](vertical-35-emit-tika-child-eml.md), and method-`5` payload evidence in [Vertical 36](vertical-36-materialise-method5-eml-payload.md).",
)
replace_once(
    "docs/operations/upstream-pst-fixture-corpus.md",
    "The fixture now produces two deterministic EML files: the unchanged 17,035-byte parent with the exact DOCX and a 453-byte single-part plain-text child. The child EML is authorised through the method-`5` attachment metadata link and excludes its four raw non-markup HTML bytes. The next use of this fixture is to materialise those exact child bytes as the `message/rfc822` attachment payload, followed by complete folder/message coverage validation.",
    "The fixture now produces two deterministic EML files: the unchanged 17,035-byte parent with the exact DOCX and a 453-byte single-part plain-text child. The child EML is also published byte-for-byte as the method-`5` `message/rfc822` attachment payload at the stable attachment path. Attachment payload output is 2 files / 12,315 bytes. The next use of this fixture is complete folder/message coverage validation.",
)

replace_once(
    "docs/README.md",
    "| Tika attachment fixture | Eight messages include one linked method-`5` child, nine directly owned recipients, ten body records, the unchanged DOCX payload, the unchanged 17,035-byte parent EML, and one exact 453-byte child EML. |\n| Current milestone | Materialise the exact child EML as the method-`5` `message/rfc822` attachment payload. |",
    "| Tika attachment fixture | Eight messages include one linked method-`5` child, nine directly owned recipients, ten body records, two exact attachment payloads, the unchanged 17,035-byte parent EML, and one exact 453-byte child EML. |\n| Current milestone | Lock complete Tika folder paths, message ownership, Unicode names, and legacy Exchange evidence. |",
)
replace_once(
    "docs/README.md",
    "- [Vertical 35: Tika child plain-text EML](operations/vertical-35-emit-tika-child-eml.md)",
    "- [Vertical 35: Tika child plain-text EML](operations/vertical-35-emit-tika-child-eml.md)\n- [Vertical 36: Method-5 child EML payload](operations/vertical-36-materialise-method5-eml-payload.md)",
)

replace_once(
    "docs/changelog/unreleased.md",
    "- Policy-gated plain-text-only EML admission from authoritative attachment metadata, retaining fail-closed behaviour for unrelated unvalidated plain-only messages.",
    "- Policy-gated plain-text-only EML admission from authoritative attachment metadata, retaining fail-closed behaviour for unrelated unvalidated plain-only messages.\n- One exact 453-byte method-`5` `message/rfc822` payload, byte-identical to the standalone child EML, with stable path, key, ordinal, parent ownership and SHA-256.\n- Shared plain-text EML construction plus focused rejection tests for missing, mismatched, duplicate, nested, ambiguous-body and unsafe-header cases.",
)
replace_once(
    "docs/changelog/unreleased.md",
    "By-value attachment payload files/bytes: 1/11862",
    "Attachment payload files/bytes: 2/12315",
)
replace_once(
    "docs/changelog/unreleased.md",
    "Attachment JSONL bytes: 1358\nExtraction TAR bytes: 227840\nTotal output bytes: 272884",
    "Attachment JSONL bytes: 1240\nExtraction TAR bytes: 228864\nTotal output bytes: 273908",
)
replace_once(
    "docs/changelog/unreleased.md",
    "The method-`5` attachment belongs to `msg_c6163b9157944cc9` and links to the separately emitted child. The child owns its recipient and bodies; the parent retains only its direct recipient, original bodies, DOCX, and unchanged EML. The method-`5` archive path is metadata-only and no empty payload file is written.",
    "The method-`5` attachment belongs to `msg_c6163b9157944cc9`, links to the separately emitted child, and now publishes the exact child EML bytes at its existing archive path. The child owns its recipient and bodies; the parent retains only its direct recipient, original bodies, DOCX, and unchanged EML.",
)
replace_once(
    "docs/changelog/unreleased.md",
    "- Materialise the exact 453-byte child EML as the method-`5` `message/rfc822` attachment payload with deterministic path, hash, and ownership.",
    "- Lock complete folder and message coverage for the Tika fixture, including exact paths, ownership, Unicode names, and legacy Exchange evidence.",
)
replace_once(
    "docs/changelog/unreleased.md",
    "- The Tika fixture has exact parent and child EMLs, but the child bytes are not yet published as the method-`5` attachment payload.\n- One method-`5` child layout is validated; method-`5` payload materialisation, nested child attachments, recursion, and broad layout coverage remain deferred.",
    "- The Tika fixture has exact parent and child EMLs plus one exact method-`5` payload, but broader producer/layout coverage remains unproven.\n- One method-`5` child layout is validated; nested child attachments, recursion, and broad layout coverage remain deferred.",
)
