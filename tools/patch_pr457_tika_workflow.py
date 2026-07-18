from pathlib import Path

path = Path(".github/workflows/tika-attachment-fixture.yml")
text = path.read_text()

old_selection = """          if len(eml_files) != 1:
              raise SystemExit(f'unexpected EML file count: {len(eml_files)}')
          eml_path = eml_files[0]
          if eml_path.name != 'msg_c6163b9157944cc9.eml':
              raise SystemExit(f'unexpected EML filename: {eml_path.name}')
          eml_bytes = eml_path.read_bytes()
"""
new_selection = """          if len(eml_files) != 2:
              raise SystemExit(f'unexpected EML file count: {len(eml_files)}')
          eml_by_name = {path.name: path for path in eml_files}
          expected_eml_names = {
              'msg_c6163b9157944cc9.eml',
              'msg_0ff529af59d373d5.eml',
          }
          if set(eml_by_name) != expected_eml_names:
              raise SystemExit(f'unexpected EML filenames: {sorted(eml_by_name)}')
          eml_path = eml_by_name['msg_c6163b9157944cc9.eml']
          eml_bytes = eml_path.read_bytes()
"""

outer_anchor = """          if eml_attachment.get_payload(decode=True) != payload:
              raise SystemExit('EML attachment payload differs from extracted DOCX bytes')
"""
child_checks = outer_anchor + """

          child_eml_path = eml_by_name['msg_0ff529af59d373d5.eml']
          child_eml_bytes = child_eml_path.read_bytes()
          if len(child_eml_bytes) != 453:
              raise SystemExit(f'unexpected child EML size: {len(child_eml_bytes)}')
          if hashlib.sha256(child_eml_bytes).hexdigest() != (
              '86ffe5567da7aa505b8be16400889170ca583fd247cc0758f00a43c2a8a99420'
          ):
              raise SystemExit('child EML checksum mismatch')
          if b'\n' in child_eml_bytes.replace(b'\r\n', b'') or b'\r' in child_eml_bytes.replace(b'\r\n', b''):
              raise SystemExit('child EML contains non-CRLF line endings')
          child_eml = BytesParser(policy=policy.default).parsebytes(child_eml_bytes)
          if child_eml.get_content_type() != 'text/plain' or child_eml.is_multipart():
              raise SystemExit(f'unexpected child EML MIME shape: {child_eml.get_content_type()}')
          if child_eml['Subject'] != 'First email':
              raise SystemExit(f'unexpected child EML subject: {child_eml["Subject"]!r}')
          if str(child_eml['To']) != '"\'lfcnassif@gmail.com\'" <lfcnassif@gmail.com>':
              raise SystemExit(f'unexpected child EML recipient: {child_eml["To"]!r}')
          if child_eml['Date'].datetime.isoformat() != '2020-11-26T22:18:00+00:00':
              raise SystemExit(f'unexpected child EML Date: {child_eml["Date"]!r}')
          if child_eml['Message-ID'] != '<3148510c2360443396a78d35e0888de9@pf.gov.br>':
              raise SystemExit(f'unexpected child EML Message-ID: {child_eml["Message-ID"]!r}')
          if b'From: Luis Filipe da Cruz Nassif </o=PF/' not in child_eml_bytes:
              raise SystemExit('child native Exchange sender address was not preserved')
          if not child_eml_bytes.endswith(b'\r\n\r\nDocx file attached.\r\n\r\n'):
              raise SystemExit('child EML plain body changed')
          if b'Content-Type: text/html' in child_eml_bytes or b'\x7f\x83\x00\x00' in child_eml_bytes:
              raise SystemExit('child raw HTML evidence leaked into MIME output')
          if b'multipart/' in child_eml_bytes or b'pstd-alternative-' in child_eml_bytes:
              raise SystemExit('child EML must remain single-part plain text')
"""

old_metrics = """              'eml_files': 1,
              'eml_bytes': 17035,
"""
new_metrics = """              'eml_files': 2,
              'eml_bytes': 17488,
"""

replacements = [
    (old_selection, new_selection, "EML selection"),
    (outer_anchor, child_checks, "child EML checks"),
    (old_metrics, new_metrics, "EML metrics"),
]

for old, new, label in replacements:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label} expected exactly one match, found {count}")
    text = text.replace(old, new)

path.write_text(text)
