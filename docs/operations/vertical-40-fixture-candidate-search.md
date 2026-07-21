# Vertical 40 Unicode attachment fixture candidate search

_Last reviewed: 21 July 2026._

## Objective

Reduce uncertainty around the next dependency-free Unicode email vertical by recording the fixture search performed after the approved corpus gap was identified.

## Search result

A public search for PST fixtures with inline attachments, Content-ID evidence, multiple attachments, or a second by-value storage layout did not identify a candidate that can yet be admitted.

The most relevant public results were parser repositories or standalone EML fixtures rather than redistributable PST bytes with immutable provenance and exact licensing. A standalone EML containing an inline image is useful as MIME reference material, but it cannot validate PST storage, ownership, attachment Property Contexts, Content-ID properties, or HTML `cid:` linkage inside a PST.

The `pst-format/libpst` repository is GPL-2.0 licensed, but locating a licensed parser repository does not establish that a particular PST binary exists, is redistributable, or exposes the required attachment evidence. PSTD must not add libpst or another parser as a dependency.

## Admission gate remains

A candidate is not approved until all of the following are recorded:

- immutable upstream repository revision and exact fixture path;
- explicit redistribution basis covering the PST bytes;
- exact byte length and SHA-256;
- NDB version, PST variant, and crypt method;
- exact folder, message, recipient, body, attachment, and EML counts from PSTD;
- exact ownership and ordinals for every candidate attachment;
- exact Content-ID and matching HTML `cid:` evidence for inline cases;
- repeated-run deterministic structured output and EML bytes;
- explicit unsupported, ambiguous, corrupt, encrypted, and non-mail object counts.

## Decision

Do not add a fixture, parser rule, MIME rule, or attachment abstraction from the current search results. Continue fixture qualification only when a candidate satisfies the provenance and redistribution gate. Until then, the highest-value safe work is maintaining exact compatibility documentation and testing PSTD against user-supplied private copies without committing those files.

## Next action

Prefer a controlled synthetic Unicode PST only if it can be generated reproducibly without introducing a production dependency and independently validated at the byte-structure level. Otherwise continue searching public repositories for explicitly licensed PST fixture bytes containing multiple by-value attachments or inline Content-ID evidence.
