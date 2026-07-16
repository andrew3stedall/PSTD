import re
from pathlib import Path


def sub_once(text: str, pattern: str, replacement: str, label: str) -> str:
    updated, count = re.subn(pattern, replacement, text, count=1, flags=re.DOTALL)
    if count != 1:
        raise SystemExit(f"{label}: expected one structural match, found {count}")
    return updated


data_path = Path("src/pst/data_tree.rs")
data = data_path.read_text(encoding="utf-8")
data = sub_once(
    data,
    r'''    if declared_total_bytes != expected_size \{.*?\n    \}\n    if declared_total_bytes > limits\.max_block_bytes \{''',
    '''    let metadata_size_status = if declared_total_bytes == expected_size {
        "metadata_size_matched"
    } else {
        "metadata_size_differs_from_xblock_total"
    };
    if declared_total_bytes > limits.max_block_bytes {''',
    "metadata size boundary",
)
data = data.replace(
    '"unicode_xblock_payload_loaded; root_bid=0x{:x}; child_blocks={}; total_bytes={declared_total_bytes}; zip_signature=504b0304",',
    '"unicode_xblock_payload_loaded; root_bid=0x{:x}; child_blocks={}; total_bytes={declared_total_bytes}; attachment_size_property={expected_size}; {metadata_size_status}; zip_signature=504b0304",',
    1,
)
if "metadata_size_differs_from_xblock_total" not in data:
    raise SystemExit("data-tree status replacement failed")
data = sub_once(
    data,
    r'''    #\[test\]\n    fn rejects_size_mismatch_and_non_docx_payload\(\) \{.*?\n    \}\n\n    #\[test\]''',
    '''    #[test]
    fn preserves_metadata_size_difference_and_rejects_non_docx_payload() {
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

    #[test]''',
    "metadata size test",
)
data_path.write_text(data, encoding="utf-8")

engine_path = Path("src/engine/metadata.rs")
engine = engine_path.read_text(encoding="utf-8")
needle = '''                            attachment_table_parse_errors += attachment_report.parse_error_count;
                            let triage_report = triage_observed_attachment_layouts(
'''
replacement = '''                            attachment_table_parse_errors += attachment_report.parse_error_count;
                            if attachment_property_report.filename_record_count > 0
                                && loaded_attachments.is_empty()
                            {
                                unavailable_attachment_records.clear();
                            }
                            let triage_report = triage_observed_attachment_layouts(
'''
if engine.count(needle) != 1:
    raise SystemExit(f"table fallback boundary: expected one match, found {engine.count(needle)}")
engine_path.write_text(engine.replace(needle, replacement, 1), encoding="utf-8")
