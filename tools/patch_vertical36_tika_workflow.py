from pathlib import Path

path = Path(".github/workflows/tika-attachment-fixture.yml")
text = path.read_text()

old_embedded = r'''          embedded_attachment = attachments[1]
          expected_embedded_attachment = {
              'message_key': 'msg_c6163b9157944cc9',
              'attachment_key': 'att_a9c94a13d70f1cb3',
              'filename_original': None,
              'filename_safe': 'attachment_1.eml',
              'content_type': None,
              'extension': 'eml',
              'size_bytes': 0,
              'declared_size_bytes': 24308,
              'size_status': 'payload_unavailable_declared_size_present',
              'sha256': 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
              'is_inline': False,
              'content_id': None,
              'attachment_method': 5,
              'embedded_message_key': 'msg_0ff529af59d373d5',
              'ordinal': 1,
              'archive_path': (
                  'attachments/msg_c6163b9157944cc9/'
                  'att_a9c94a13d70f1cb3_attachment_1.eml'
              ),
              'extraction_status': (
                  'embedded_message_metadata_extracted; '
                  'embedded_message_key=msg_0ff529af59d373d5; '
                  'data_nid=0x00200104; data_bid=0x670; subnode_bid=0x67a'
              ),
          }
          if embedded_attachment != expected_embedded_attachment:
              raise SystemExit(
                  f'unexpected embedded attachment: {embedded_attachment!r}'
              )
          if (archive / embedded_attachment['archive_path']).exists():
              raise SystemExit('metadata-only embedded attachment must not create a payload file')
'''
new_embedded = r'''          embedded_attachment = attachments[1]
          expected_embedded_attachment = {
              'message_key': 'msg_c6163b9157944cc9',
              'attachment_key': 'att_a9c94a13d70f1cb3',
              'filename_original': None,
              'filename_safe': 'attachment_1.eml',
              'content_type': 'message/rfc822',
              'extension': 'eml',
              'size_bytes': 453,
              'declared_size_bytes': 24308,
              'size_status': 'size_mismatch',
              'sha256': '86ffe5567da7aa505b8be16400889170ca583fd247cc0758f00a43c2a8a99420',
              'is_inline': False,
              'content_id': None,
              'attachment_method': 5,
              'embedded_message_key': 'msg_0ff529af59d373d5',
              'ordinal': 1,
              'archive_path': (
                  'attachments/msg_c6163b9157944cc9/'
                  'att_a9c94a13d70f1cb3_attachment_1.eml'
              ),
              'extraction_status': 'extracted_embedded_message_eml',
          }
          if embedded_attachment != expected_embedded_attachment:
              raise SystemExit(
                  f'unexpected embedded attachment: {embedded_attachment!r}'
              )
          embedded_payload_path = archive / embedded_attachment['archive_path']
          if not embedded_payload_path.is_file():
              raise SystemExit('embedded message EML payload file is absent')
          embedded_payload = embedded_payload_path.read_bytes()
          if len(embedded_payload) != 453:
              raise SystemExit(f'unexpected embedded payload size: {len(embedded_payload)}')
          if hashlib.sha256(embedded_payload).hexdigest() != embedded_attachment['sha256']:
              raise SystemExit('embedded payload checksum mismatch')
'''

old_child_tail = r'''          if b'multipart/' in child_eml_bytes or b'pstd-alternative-' in child_eml_bytes:
              raise SystemExit('child EML must remain single-part plain text')
'''
new_child_tail = old_child_tail + r'''          if embedded_payload != child_eml_bytes:
              raise SystemExit('method-5 payload differs from standalone child EML')
'''

old_metrics = r'''              'attachment_payload_files': 1,
              'attachment_payload_bytes': 11862,
              'eml_files': 2,
              'eml_bytes': 17488,
              'messages_jsonl_bytes': 23086,
              'bodies_jsonl_bytes': 2820,
              'recipients_jsonl_bytes': 2708,
              'attachments_jsonl_bytes': 1358,
              'tar_bytes': 227840,
              'output_total_bytes': 272884,
'''
new_metrics = r'''              'attachment_payload_files': 2,
              'attachment_payload_bytes': 12315,
              'eml_files': 2,
              'eml_bytes': 17488,
              'messages_jsonl_bytes': 23086,
              'bodies_jsonl_bytes': 2820,
              'recipients_jsonl_bytes': 2708,
              'attachments_jsonl_bytes': 1240,
              'tar_bytes': 228864,
              'output_total_bytes': 273908,
'''

replacements = [
    (old_embedded, new_embedded, "embedded attachment contract"),
    (old_child_tail, new_child_tail, "payload byte identity"),
    (old_metrics, new_metrics, "fixture metrics"),
]

for old, new, label in replacements:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    text = text.replace(old, new, 1)

path.write_text(text)
