from pathlib import Path


source = Path("src/bin/pstd-eml.rs")
text = source.read_text(encoding="utf-8")

old_branch = '''    if attachments.is_empty() {
        let html = html?;
        push_header(
            &mut eml,
            "Content-Type",
            &format!("multipart/alternative; boundary=\\\"{ALTERNATIVE_BOUNDARY}\\\""),
        );
        eml.push_str("\\r\\n");
        push_alternative_body(&mut eml, text, html);
    } else {
'''
new_branch = '''    if attachments.is_empty() {
        if let Some(html) = html {
            push_header(
                &mut eml,
                "Content-Type",
                &format!("multipart/alternative; boundary=\\\"{ALTERNATIVE_BOUNDARY}\\\""),
            );
            eml.push_str("\\r\\n");
            push_alternative_body(&mut eml, text, html);
        } else {
            push_header(&mut eml, "Content-Type", "text/plain; charset=utf-8");
            push_header(&mut eml, "Content-Transfer-Encoding", "8bit");
            eml.push_str("\\r\\n");
            eml.push_str(&normalize_crlf(text));
            if !eml.ends_with("\\r\\n") {
                eml.push_str("\\r\\n");
            }
        }
    } else {
'''
if text.count(old_branch) != 1:
    raise SystemExit("attachmentless EML branch did not match exactly once")
text = text.replace(old_branch, new_branch)

old_test = '''    #[test]
    fn fails_closed_for_missing_html_and_boundary_collision() {
        let recipients = vec![recipient(0, "to")];
        let missing = MessageBodies {
            text: Some(b"plain".to_vec()),
            html: None,
        };
        assert!(build_eml(&message(), &recipients, &missing, &[]).is_none());
        let collision = MessageBodies {
            text: Some(ALTERNATIVE_BOUNDARY.as_bytes().to_vec()),
            html: Some("<b>rich</b>".to_string()),
        };
        assert!(build_eml(&message(), &recipients, &collision, &[]).is_none());
    }
'''
new_test = '''    #[test]
    fn emits_single_part_plain_text_without_html_or_attachments() {
        let bodies = MessageBodies {
            text: Some(b"plain\\nbody".to_vec()),
            html: None,
        };
        let eml = build_eml(&message(), &[recipient(0, "to")], &bodies, &[]).unwrap();
        let eml = String::from_utf8(eml).unwrap();

        assert!(eml.contains("Content-Type: text/plain; charset=utf-8\\r\\n"));
        assert!(eml.contains("Content-Transfer-Encoding: 8bit\\r\\n"));
        assert!(eml.contains("\\r\\n\\r\\nplain\\r\\nbody\\r\\n"));
        assert!(!eml.contains("multipart/alternative"));
        assert!(!eml.contains(ALTERNATIVE_BOUNDARY));
        assert!(!eml.contains("Content-Type: text/html"));
    }

    #[test]
    fn fails_closed_for_boundary_collision() {
        let recipients = vec![recipient(0, "to")];
        let collision = MessageBodies {
            text: Some(ALTERNATIVE_BOUNDARY.as_bytes().to_vec()),
            html: Some("<b>rich</b>".to_string()),
        };
        assert!(build_eml(&message(), &recipients, &collision, &[]).is_none());
    }
'''
if text.count(old_test) != 1:
    raise SystemExit("attachmentless EML test block did not match exactly once")
source.write_text(text.replace(old_test, new_test), encoding="utf-8")

workflow = Path(".github/workflows/tika-attachment-fixture.yml")
fixture = workflow.read_text(encoding="utf-8")

old_file_selection = '''          if len(eml_files) != 1:
              raise SystemExit(f'unexpected EML file count: {len(eml_files)}')
          eml_path = eml_files[0]
          if eml_path.name != 'msg_c6163b9157944cc9.eml':
              raise SystemExit(f'unexpected EML filename: {eml_path.name}')
'''
new_file_selection = '''          if len(eml_files) != 2:
              raise SystemExit(f'unexpected EML file count: {len(eml_files)}')
          eml_by_name = {path.name: path for path in eml_files}
          expected_eml_names = {
              'msg_c6163b9157944cc9.eml',
              'msg_0ff529af59d373d5.eml',
          }
          if set(eml_by_name) != expected_eml_names:
              raise SystemExit(f'unexpected EML filenames: {sorted(eml_by_name)}')
          eml_path = eml_by_name['msg_c6163b9157944cc9.eml']
'''
if fixture.count(old_file_selection) != 1:
    raise SystemExit("Tika EML file selection did not match exactly once")
fixture = fixture.replace(old_file_selection, new_file_selection)

outer_anchor = '''          if eml_attachment.get_payload(decode=True) != payload:
              raise SystemExit('EML attachment payload differs from extracted DOCX bytes')
'''
child_checks = outer_anchor + '''

          child_eml_path = eml_by_name['msg_0ff529af59d373d5.eml']
          child_eml_bytes = child_eml_path.read_bytes()
          if len(child_eml_bytes) != 453:
              raise SystemExit(f'unexpected child EML size: {len(child_eml_bytes)}')
          if hashlib.sha256(child_eml_bytes).hexdigest() != (
              '86ffe5567da7aa505b8be16400889170ca583fd247cc0758f00a43c2a8a99420'
          ):
              raise SystemExit('child EML checksum mismatch')
          if b'\\n' in child_eml_bytes.replace(b'\\r\\n', b'') or b'\\r' in child_eml_bytes.replace(b'\\r\\n', b''):
              raise SystemExit('child EML contains non-CRLF line endings')
          child_eml = BytesParser(policy=policy.default).parsebytes(child_eml_bytes)
          if child_eml.get_content_type() != 'text/plain' or child_eml.is_multipart():
              raise SystemExit(f'unexpected child EML MIME shape: {child_eml.get_content_type()}')
          if child_eml['Subject'] != 'First email':
              raise SystemExit(f'unexpected child EML subject: {child_eml["Subject"]!r}')
          if str(child_eml['To']) != '"\\'lfcnassif@gmail.com\\'" <lfcnassif@gmail.com>':
              raise SystemExit(f'unexpected child EML recipient: {child_eml["To"]!r}')
          if child_eml['Date'].datetime.isoformat() != '2020-11-26T22:18:00+00:00':
              raise SystemExit(f'unexpected child EML Date: {child_eml["Date"]!r}')
          if child_eml['Message-ID'] != '<3148510c2360443396a78d35e0888de9@pf.gov.br>':
              raise SystemExit(f'unexpected child EML Message-ID: {child_eml["Message-ID"]!r}')
          if b'From: Luis Filipe da Cruz Nassif </o=PF/' not in child_eml_bytes:
              raise SystemExit('child native Exchange sender address was not preserved')
          if not child_eml_bytes.endswith(b'\\r\\n\\r\\nDocx file attached.\\r\\n\\r\\n'):
              raise SystemExit('child EML plain body changed')
          if b'Content-Type: text/html' in child_eml_bytes or b'\\x7f\\x83\\x00\\x00' in child_eml_bytes:
              raise SystemExit('child raw HTML evidence leaked into MIME output')
          if b'multipart/' in child_eml_bytes or b'pstd-alternative-' in child_eml_bytes:
              raise SystemExit('child EML must remain single-part plain text')
'''
if fixture.count(outer_anchor) != 1:
    raise SystemExit("Tika outer EML anchor did not match exactly once")
fixture = fixture.replace(outer_anchor, child_checks)

old_metrics = "              'eml_files': 1,\n              'eml_bytes': 17035,"
new_metrics = "              'eml_files': 2,\n              'eml_bytes': 17488,"
if fixture.count(old_metrics) != 1:
    raise SystemExit("Tika EML metric block did not match exactly once")
workflow.write_text(fixture.replace(old_metrics, new_metrics), encoding="utf-8")
