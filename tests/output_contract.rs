use pstd::output::ids;
use pstd::output::jsonl_writer::JsonlBuffer;
use pstd::output::paths::{sanitize_segment, UniquePathTracker};
use pstd::output::tar_writer::TarShardWriter;

#[test]
fn stable_ids_are_deterministic() {
    let first = ids::message_key("pst_abc", "message-1");
    let second = ids::message_key("pst_abc", "message-1");
    assert_eq!(first, second);
    assert!(first.starts_with("msg_"));
}

#[test]
fn path_sanitizer_blocks_unsafe_segments() {
    let sanitized = sanitize_segment("../../evil:name?.txt");
    assert!(!sanitized.contains("/"));
    assert!(!sanitized.contains("\\"));
    assert!(!sanitized.contains(":"));
    assert!(!sanitized.contains("?"));
}

#[test]
fn unique_path_tracker_disambiguates_duplicate_names() {
    let mut tracker = UniquePathTracker::default();
    assert_eq!(tracker.unique_file_name("invoice.pdf"), "invoice.pdf");
    assert_eq!(tracker.unique_file_name("invoice.pdf"), "invoice_0002.pdf");
}

#[test]
fn jsonl_writer_outputs_one_json_object_per_line() {
    let mut writer = JsonlBuffer::new();
    writer.write_record(&serde_json::json!({"a": 1})).unwrap();
    writer.write_record(&serde_json::json!({"b": 2})).unwrap();
    let bytes = writer.into_bytes();
    let text = String::from_utf8(bytes).unwrap();
    let lines: Vec<_> = text.lines().collect();
    assert_eq!(lines.len(), 2);
    for line in lines {
        let parsed: serde_json::Value = serde_json::from_str(line).unwrap();
        assert!(parsed.is_object());
    }
}

#[test]
fn tar_writer_creates_readable_archive() {
    let temp = tempfile::tempdir().unwrap();
    let mut writer = TarShardWriter::new(temp.path(), "pst_test", 1024 * 1024).unwrap();
    writer.append_bytes(&["data", "messages.jsonl"], b"{}\n").unwrap();
    let shards = writer.finish().unwrap();
    assert_eq!(shards.len(), 1);
    assert!(shards[0].path.exists());
}
