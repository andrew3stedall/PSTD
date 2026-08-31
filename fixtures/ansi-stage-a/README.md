# ANSI Stage-A fixture manifest

name: pstd-ansi-stage-a.pst
format: Microsoft Outlook PST ANSI
ndb_version: 14
generator:
  source: tools/ansi_fixture.rs
  repository_commit: 8ab19f0d67b4a9ae9a546777d918569f16d909f6
  platform: Linux Rust 1.98 stable
content: controlled synthetic empty store only
licence: CC0-1.0
byte_length: 2048
sha256: b5de1ce4cebacc2ea4cefddb4ab9c4d32e5fed04b81cd681e8831faf1323c765
header:
  magic: "!BDN"
  client_signature: "SM"
  version: 14
  client_version: 19
  crypt_method: 0
roots:
  bbt_offset: 1024
  nbt_offset: 1536
  bbt_entries: 0
  nbt_entries: 0
validation:
  independent_bytes: scripts/validate_ansi_stage_a.py
  pstd: "ansi_pst; partial; allows_extraction=false; bbt_entries=0; nbt_entries=0"
  independent_reader: "libpff pffinfo 20180714 (Ubuntu pff-tools); exit 0"
  workflow: .github/workflows/ansi-stage-a.yml
scope: Stage A structural baseline only; no ANSI email compatibility claim

## Reproduce

```bash
rustc --edition=2021 --deny warnings tools/ansi_fixture.rs -o /tmp/ansi-fixture
/tmp/ansi-fixture /tmp/pstd-ansi-stage-a.pst 0
# Existing outputs are protected; use --force only for an intentional replacement.
python3 scripts/validate_ansi_stage_a.py /tmp/pstd-ansi-stage-a.pst 0
sha256sum /tmp/pstd-ansi-stage-a.pst
pffinfo /tmp/pstd-ansi-stage-a.pst
```

The workflow also emits crypt-method 2 and unknown-method 7 derivatives. Method
2 remains classified as supported-but-empty and method 7 is rejected as
unsupported with extraction disabled. The fixture contains no folders, messages,
bodies, recipients, attachments, typed objects or EML outputs.
