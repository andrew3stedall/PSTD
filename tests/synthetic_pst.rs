//! Extracts a generated mailbox and checks the result against the generator's manifest.
//!
//! The emitter in `tools/synthetic_pst.rs` is written from the MS-PST layouts and
//! shares no code with the reader, so these tests compare two independent
//! implementations. Reader gaps that are known and tracked are listed in the
//! `*_GAPS` constants with their issue; everything else must match exactly.

#[path = "../tools/synthetic_pst.rs"]
mod synthetic_pst;

use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::path::Path;

use serde_json::Value;
use synthetic_pst::{fnv1a64, Config, Fault, Manifest, ManifestMessage, Scenario};

/// Bodies larger than one heap allocation live in a subnode stream (issue #622).
const BODY_GAPS: &[Scenario] = &[Scenario::LargeBody];
/// Recipient rows too large for one heap allocation live in a subnode (issue #623).
const RECIPIENT_GAPS: &[Scenario] = &[Scenario::ManyRecipients];
/// An empty attachment value is emitted as four literal bytes (issue #622).
const INTACT_ATTACHMENT_GAPS: &[Scenario] = &[Scenario::ZeroLengthAttachment];

const MESSAGES: usize = 72;
const SEED: u64 = 7;

struct Extraction {
    members: HashMap<String, Vec<u8>>,
}

impl Extraction {
    fn rows(&self, name: &str) -> Vec<Value> {
        let Some(bytes) = self.members.get(&format!("data/{name}.jsonl")) else {
            return Vec::new();
        };
        std::str::from_utf8(bytes)
            .expect("jsonl is utf-8")
            .lines()
            .map(|line| serde_json::from_str(line).expect("jsonl row parses"))
            .collect()
    }

    fn payload(&self, archive_path: Option<&str>) -> Option<&[u8]> {
        self.members.get(archive_path?).map(Vec::as_slice)
    }
}

fn generate(dir: &Path, name: &str) -> (std::path::PathBuf, Manifest) {
    let path = dir.join(name);
    let mut bytes = Cursor::new(Vec::new());
    let manifest = synthetic_pst::generate(
        &mut bytes,
        Config {
            seed: SEED,
            messages: MESSAGES,
        },
    )
    .expect("generation succeeds");
    std::fs::write(&path, bytes.into_inner()).expect("fixture is written");
    (path, manifest)
}

fn extract(input: &Path, output: &Path) -> Extraction {
    let config = pstd::config::ExtractConfig {
        input: input.to_path_buf(),
        output: output.to_path_buf(),
        continue_on_error: false,
        overwrite: true,
        manifest_only: false,
        archive_format: "tar".to_string(),
        data_format: "jsonl".to_string(),
        tar_shard_size_mb: 1024,
        progress: "none".to_string(),
        log_level: "info".to_string(),
        profile: "balanced".to_string(),
        readpst: Default::default(),
    };
    pstd::engine::runner::run_extract(config).expect("extraction of a damaged mailbox succeeds");

    let mut members = HashMap::new();
    let mut shards: Vec<_> = std::fs::read_dir(output.join("archives"))
        .expect("archives directory exists")
        .map(|entry| entry.expect("directory entry").path())
        .collect();
    shards.sort();
    for shard in shards {
        let mut archive = tar::Archive::new(std::fs::File::open(shard).expect("shard opens"));
        for entry in archive.entries().expect("tar entries") {
            let mut entry = entry.expect("tar entry");
            let path = entry
                .path()
                .expect("entry path")
                .to_string_lossy()
                .into_owned();
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).expect("entry bytes");
            members.insert(path, bytes);
        }
    }
    Extraction { members }
}

fn text(value: &Value, field: &str) -> String {
    value[field].as_str().unwrap_or("").to_string()
}

fn leaf(folder: &str) -> &str {
    folder.rsplit('/').next().unwrap_or(folder)
}

#[test]
fn generation_is_deterministic_and_covers_every_scenario() {
    let dir = tempfile::tempdir().unwrap();
    let (first, manifest) = generate(dir.path(), "first.pst");
    let (second, _) = generate(dir.path(), "second.pst");
    let first = std::fs::read(first).unwrap();
    assert_eq!(first, std::fs::read(second).unwrap());
    assert_eq!(first.len() as u64, manifest.file_bytes);
    assert_eq!(&first[..4], b"!BDN");

    for scenario in synthetic_pst::SPECIAL_SCENARIOS {
        assert!(
            manifest
                .messages
                .iter()
                .any(|message| message.scenario == *scenario),
            "scenario {} is not generated",
            scenario.label()
        );
    }
    assert!(manifest.messages.iter().any(|message| message.draft));
    assert!(manifest.messages.iter().any(|message| message.deleted));
    let json = manifest.to_json();
    let parsed: Value = serde_json::from_str(&json).expect("manifest is valid JSON");
    assert_eq!(parsed["messages"].as_array().unwrap().len(), MESSAGES);
}

#[test]
fn extraction_matches_the_manifest_and_contains_every_fault() {
    let dir = tempfile::tempdir().unwrap();
    let (input, manifest) = generate(dir.path(), "mailbox.pst");
    let extraction = extract(&input, &dir.path().join("out"));

    let messages: HashMap<String, Value> = extraction
        .rows("messages")
        .into_iter()
        .filter(|row| row["message_node_id"].is_string())
        .map(|row| (text(&row, "message_node_id"), row))
        .collect();
    let mut bodies: HashMap<String, Vec<Value>> = HashMap::new();
    for row in extraction.rows("bodies") {
        bodies
            .entry(text(&row, "message_key"))
            .or_default()
            .push(row);
    }
    let mut recipients: HashMap<String, usize> = HashMap::new();
    for row in extraction.rows("recipients") {
        *recipients.entry(text(&row, "message_key")).or_default() += 1;
    }
    let mut attachments: HashMap<String, Vec<Value>> = HashMap::new();
    for row in extraction.rows("attachments") {
        attachments
            .entry(text(&row, "message_key"))
            .or_default()
            .push(row);
    }

    // Every folder is discovered. The reader does not yet rebuild nesting, so folders
    // are compared by name.
    let folder_names: Vec<String> = extraction
        .rows("folders")
        .iter()
        .map(|row| leaf(&text(row, "folder_path")).to_string())
        .collect();
    for folder in &manifest.folders {
        assert!(
            folder_names.iter().any(|name| name == leaf(folder)),
            "folder {folder} was not discovered"
        );
    }

    // A contents-table row without a node must not invent a message.
    for ghost in &manifest.dangling_contents_rows {
        assert!(!messages.contains_key(&format!("node_{ghost:x}")));
    }

    for expected in &manifest.messages {
        let context = format!("message {} ({})", expected.index, expected.scenario.label());
        let node = format!("node_{:x}", expected.node_id);
        let row = messages
            .get(&node)
            .unwrap_or_else(|| panic!("{context}: no message record"));
        let key = text(row, "message_key");

        if !expected.recoverable() {
            // The node is unreadable: nothing may be presented as decoded content.
            assert!(
                row["subject"].is_null(),
                "{context}: subject from unreadable node"
            );
            assert!(
                row["sender_email"].is_null(),
                "{context}: sender from unreadable node"
            );
            let leaked = bodies.get(&key).is_some_and(|rows| {
                rows.iter()
                    .any(|body| extraction.payload(body["archive_path"].as_str()).is_some())
            });
            assert!(!leaked, "{context}: body payload from unreadable node");
            continue;
        }

        assert_eq!(text(row, "subject"), expected.subject, "{context}: subject");
        assert_eq!(
            text(row, "message_class"),
            expected.message_class,
            "{context}: class"
        );
        assert_eq!(
            text(row, "internet_message_id"),
            expected.internet_message_id,
            "{context}: message id"
        );
        if expected.in_contents_table {
            assert_eq!(
                leaf(&text(row, "folder_path")),
                leaf(&expected.folder),
                "{context}: folder"
            );
        }
        if expected.draft {
            let flags: u32 = text(row, "message_flags")
                .parse()
                .expect("flags are numeric");
            assert_ne!(flags & 0x08, 0, "{context}: draft keeps the unsent flag");
            assert!(
                row["sender_email"].is_null(),
                "{context}: draft has no sender"
            );
        }

        check_body(&extraction, &bodies, &key, expected, &context);

        if !RECIPIENT_GAPS.contains(&expected.scenario) {
            assert_eq!(
                recipients.get(&key).copied().unwrap_or(0),
                expected.recipient_count,
                "{context}: recipients"
            );
        }

        check_attachments(&extraction, attachments.get(&key), expected, &context);
    }
}

fn check_body(
    extraction: &Extraction,
    bodies: &HashMap<String, Vec<Value>>,
    key: &str,
    expected: &ManifestMessage,
    context: &str,
) {
    let rows = bodies.get(key).map(Vec::as_slice).unwrap_or(&[]);
    let payload = |kind: &str| {
        rows.iter()
            .filter(|row| row["body_type"] == kind)
            .find_map(|row| extraction.payload(row["archive_path"].as_str()))
    };
    if !expected.body_text.is_empty() && !BODY_GAPS.contains(&expected.scenario) {
        let text = payload("text").unwrap_or_else(|| panic!("{context}: no text body payload"));
        assert_eq!(
            fnv1a64(text),
            fnv1a64(expected.body_text.as_bytes()),
            "{context}: text body bytes"
        );
    }
    if expected.has_html {
        assert!(payload("html").is_some(), "{context}: no html body payload");
    }
}

fn check_attachments(
    extraction: &Extraction,
    rows: Option<&Vec<Value>>,
    expected: &ManifestMessage,
    context: &str,
) {
    let rows = rows.map(Vec::as_slice).unwrap_or(&[]);
    for attachment in &expected.attachments {
        let intact = attachment.fault == Fault::None
            && !attachment.dangling_row
            && !attachment.embedded_message;
        if !intact || INTACT_ATTACHMENT_GAPS.contains(&expected.scenario) {
            // A damaged attachment must not stop its message or its siblings; what the
            // reader reports for the damaged item itself is tracked separately.
            continue;
        }
        let row = rows
            .iter()
            .find(|row| row["filename_original"] == attachment.filename.as_str())
            .unwrap_or_else(|| panic!("{context}: no record for {}", attachment.filename));
        let bytes = extraction
            .payload(row["archive_path"].as_str())
            .unwrap_or_else(|| panic!("{context}: no payload for {}", attachment.filename));
        assert_eq!(
            bytes.len(),
            attachment.size,
            "{context}: {} size",
            attachment.filename
        );
        assert_eq!(
            fnv1a64(bytes),
            attachment.fnv1a64,
            "{context}: {} bytes",
            attachment.filename
        );
    }
}
