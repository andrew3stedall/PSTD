from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


data_path = Path("src/pst/data_tree.rs")
data = data_path.read_text(encoding="utf-8")
data = replace_once(
    data,
    '''    if declared_total_bytes != expected_size {
        return Err(PstdError::pst_parse(
            Some(root.block_ref.offset.0),
            format!(
                "XBLOCK declared total {declared_total_bytes} does not match expected attachment size {expected_size}"
            ),
        ));
    }
''',
    '''    let metadata_size_status = if declared_total_bytes == expected_size {
        "metadata_size_matched"
    } else {
        "metadata_size_differs_from_xblock_total"
    };
''',
    "metadata size boundary",
)
data = replace_once(
    data,
    '''            "unicode_xblock_payload_loaded; root_bid=0x{:x}; child_blocks={}; total_bytes={declared_total_bytes}; zip_signature=504b0304",
            root_bid.0,
            child_bids.len()
''',
    '''            "unicode_xblock_payload_loaded; root_bid=0x{:x}; child_blocks={}; total_bytes={declared_total_bytes}; attachment_size_property={expected_size}; {metadata_size_status}; zip_signature=504b0304",
            root_bid.0,
            child_bids.len()
''',
    "data tree status",
)
data = replace_once(
    data,
    '''    fn rejects_size_mismatch_and_non_docx_payload() {
        let root = xblock(&[0x640], 4);
        let (file, bbt) = fixture(&[(0x632, root), (0x640, b"nope".to_vec())]);
        let reader = PstByteReader::open(file.path()).unwrap();

        let mismatch =
            load_unicode_xblock_payload(&reader, &bbt, BlockId(0x632), 5, ParserLimits::default())
                .unwrap_err();
        assert!(mismatch.to_string().contains("does not match expected"));

        let signature =
            load_unicode_xblock_payload(&reader, &bbt, BlockId(0x632), 4, ParserLimits::default())
                .unwrap_err();
        assert!(signature.to_string().contains("unexpected signature"));
    }
''',
    '''    fn preserves_metadata_size_difference_and_rejects_non_docx_payload() {
        let root = xblock(&[0x640], 4);
        let (file, bbt) = fixture(&[(0x632, root), (0x640, b"PK\\x03\\x04".to_vec())]);
        let reader = PstByteReader::open(file.path()).unwrap();

        let mismatch =
            load_unicode_xblock_payload(&reader, &bbt, BlockId(0x632), 5, ParserLimits::default())
                .unwrap();
        assert_eq!(mismatch.declared_total_bytes, 4);
        assert!(mismatch
            .status
            .contains("metadata_size_differs_from_xblock_total"));

        let root = xblock(&[0x640], 4);
        let (file, bbt) = fixture(&[(0x632, root), (0x640, b"nope".to_vec())]);
        let reader = PstByteReader::open(file.path()).unwrap();
        let signature =
            load_unicode_xblock_payload(&reader, &bbt, BlockId(0x632), 4, ParserLimits::default())
                .unwrap_err();
        assert!(signature.to_string().contains("unexpected signature"));
    }
''',
    "metadata size test",
)
data_path.write_text(data, encoding="utf-8")

engine_path = Path("src/engine/metadata.rs")
engine = engine_path.read_text(encoding="utf-8")
engine = replace_once(
    engine,
    '''                            attachment_table_parse_errors += attachment_report.parse_error_count;
                            let triage_report = triage_observed_attachment_layouts(
''',
    '''                            attachment_table_parse_errors += attachment_report.parse_error_count;
                            if attachment_property_report.filename_record_count > 0
                                && loaded_attachments.is_empty()
                            {
                                unavailable_attachment_records.clear();
                            }
                            let triage_report = triage_observed_attachment_layouts(
''',
    "property context table fallback boundary",
)
engine = replace_once(
    engine,
    '''                                    "{}; {}; {}; {}; property_contexts={}; filename_records={}; rejected_contexts={}",
''',
    '''                                    "{}; {}; {}; {}; property_contexts={}; filename_records={}; rejected_contexts={}; payloads={}; payload_bytes={}; payload_failures={}",
''',
    "attachment status format",
)
engine = replace_once(
    engine,
    '''                                    attachment_property_report.rejected_context_count,
                                );
''',
    '''                                    attachment_property_report.rejected_context_count,
                                    attachment_property_report.payload_count,
                                    attachment_property_report.payload_bytes,
                                    attachment_property_report.payload_failure_count,
                                );
''',
    "attachment status values",
)
engine_path.write_text(engine, encoding="utf-8")
