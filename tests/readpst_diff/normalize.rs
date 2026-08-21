use super::manifest::{sha256_hex, ArtifactDigest, EvidenceStatus};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use tar::Archive;

#[derive(Debug, Clone)]
pub struct NormalizationLimits {
    pub max_records: usize,
    pub max_member_bytes: u64,
}

impl Default for NormalizationLimits {
    fn default() -> Self {
        Self {
            max_records: 100_000,
            max_member_bytes: 64 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NormalizedRecord {
    pub kind: String,
    pub identity: String,
    pub status: EvidenceStatus,
    pub fields: BTreeMap<String, String>,
    pub payload_hashes: Vec<String>,
    pub children: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NormalizedOutput {
    pub tool: String,
    pub status: EvidenceStatus,
    pub records: Vec<NormalizedRecord>,
    pub artifacts: Vec<ArtifactDigest>,
    pub diagnostics: Vec<String>,
}

impl NormalizedOutput {
    pub fn validate(&self) -> Result<(), String> {
        if self.tool.trim().is_empty() {
            return Err("normalized tool must not be empty".to_string());
        }
        if self.records.len() > 100_000 {
            return Err("normalized record count exceeds contract budget".to_string());
        }
        for artifact in &self.artifacts {
            artifact.validate()?;
        }
        Ok(())
    }

    pub fn sort_deterministically(&mut self) {
        self.records
            .sort_by(|left, right| (left.kind.as_str(), left.identity.as_str()).cmp(&(right.kind.as_str(), right.identity.as_str())));
        self.artifacts.sort_by(|left, right| left.path.cmp(&right.path));
        self.diagnostics.sort();
    }
}

pub fn normalize_pstd_archive(
    output_root: &Path,
    tool: impl Into<String>,
    limits: &NormalizationLimits,
) -> Result<NormalizedOutput, String> {
    let mut archives = find_files(output_root, |path| {
        path.extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("tar"))
    })?;
    archives.sort();
    if archives.is_empty() {
        return Ok(NormalizedOutput {
            tool: tool.into(),
            status: EvidenceStatus::Empty,
            records: Vec::new(),
            artifacts: Vec::new(),
            diagnostics: vec!["canonical_archive_missing".to_string()],
        });
    }

    let mut output = NormalizedOutput {
        tool: tool.into(),
        status: EvidenceStatus::Present,
        records: Vec::new(),
        artifacts: Vec::new(),
        diagnostics: Vec::new(),
    };
    for archive_path in archives {
        let archive_name = archive_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| "archive_name_not_utf8".to_string())?
            .to_string();
        let file = File::open(&archive_path)
            .map_err(|error| format!("canonical_archive_open_failed:{error}"))?;
        let mut archive = Archive::new(file);
        for entry in archive
            .entries()
            .map_err(|error| format!("canonical_archive_entries_failed:{error}"))?
        {
            let mut entry = entry.map_err(|error| format!("canonical_archive_entry_failed:{error}"))?;
            let path = entry
                .path()
                .map_err(|error| format!("canonical_archive_path_failed:{error}"))?
                .to_path_buf();
            let relative = path_string(Path::new("archive").join(&archive_name).join(&path).as_path());
            if path.is_absolute() || path.components().any(|component| component == std::path::Component::ParentDir) {
                return Err(format!("canonical_archive_path_escape: {relative}"));
            }
            let size = entry.size();
            if size > limits.max_member_bytes {
                return Err(format!("canonical_archive_member_too_large: {relative}"));
            }
            let mut bytes = Vec::with_capacity(size.min(1_048_576) as usize);
            entry
                .read_to_end(&mut bytes)
                .map_err(|error| format!("canonical_archive_member_read_failed:{error}"))?;
            if path.extension().and_then(|extension| extension.to_str()) == Some("jsonl") {
                let kind = path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("record")
                    .to_string();
                for line in bytes.split(|byte| *byte == b'\n') {
                    if line.iter().all(u8::is_ascii_whitespace) {
                        continue;
                    }
                    let value: Value = serde_json::from_slice(line)
                        .map_err(|error| format!("canonical_jsonl_parse_failed:{relative}:{error}"))?;
                    output.records.push(normalize_json_record(&kind, value)?);
                    if output.records.len() > limits.max_records {
                        return Err("normalized record count exceeds contract budget".to_string());
                    }
                }
            } else {
                let artifact = ArtifactDigest {
                    artifact_type: artifact_type(&relative),
                    path: relative,
                    sha256: sha256_hex(&bytes),
                    size_bytes: bytes.len() as u64,
                };
                artifact.validate()?;
                output.artifacts.push(artifact);
            }
        }
    }
    output.sort_deterministically();
    output.validate()?;
    Ok(output)
}

pub fn normalize_readpst_directory(
    output_root: &Path,
    tool: impl Into<String>,
    limits: &NormalizationLimits,
) -> Result<NormalizedOutput, String> {
    let mut files = find_files(output_root, |_| true)?;
    files.sort();
    let mut output = NormalizedOutput {
        tool: tool.into(),
        status: if files.is_empty() {
            EvidenceStatus::Empty
        } else {
            EvidenceStatus::Present
        },
        records: Vec::new(),
        artifacts: Vec::new(),
        diagnostics: Vec::new(),
    };
    for path in files {
        let relative = path_string(
            path.strip_prefix(output_root)
                .map_err(|_| "readpst_relative_path_failed".to_string())?,
        );
        let bytes = fs::read(&path).map_err(|error| format!("readpst_output_read_failed:{error}"))?;
        if bytes.len() as u64 > limits.max_member_bytes {
            return Err(format!("readpst_output_file_too_large: {relative}"));
        }
        let artifact = ArtifactDigest {
            artifact_type: artifact_type(&relative),
            path: relative.clone(),
            sha256: sha256_hex(&bytes),
            size_bytes: bytes.len() as u64,
        };
        artifact.validate()?;
        output.artifacts.push(artifact);
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        if extension == "eml" || extension == "msg" {
            output.records.push(normalize_message_file(&relative, &bytes));
        } else if extension == "vcf" {
            output.records.push(normalize_text_record("contact", &relative, &bytes));
        } else if extension == "ics" || extension == "jrn" {
            output.records.push(normalize_text_record("typed_item", &relative, &bytes));
        } else if extension.is_empty() && bytes.windows(5).any(|window| window == b"From ") {
            output.records.extend(normalize_mbox(&relative, &bytes));
        }
        if output.records.len() > limits.max_records {
            return Err("normalized record count exceeds contract budget".to_string());
        }
    }
    output.sort_deterministically();
    output.validate()?;
    Ok(output)
}

fn normalize_json_record(kind: &str, value: Value) -> Result<NormalizedRecord, String> {
    let object = value
        .as_object()
        .ok_or_else(|| format!("normalized_record_not_object:{kind}"))?;
    let mut fields: BTreeMap<String, String> = BTreeMap::new();
    let mut payload_hashes = Vec::new();
    let mut children = Vec::new();
    for (key, value) in object {
        if is_volatile_field(key) {
            continue;
        }
        if key.to_ascii_lowercase().contains("sha256") {
            if let Some(hash) = value.as_str() {
                payload_hashes.push(hash.to_string());
            }
        }
        if key.to_ascii_lowercase().contains("child") || key == "child_edges" {
            if let Some(array) = value.as_array() {
                children.extend(array.iter().map(canonical_value));
            } else {
                children.push(canonical_value(value));
            }
        }
        fields.insert(key.clone(), field_value(value));
    }
    payload_hashes.sort();
    children.sort();
    let identity = identity_from_fields(&fields)
        .unwrap_or_else(|| sha256_hex(canonical_json(&value).as_bytes()));
    let status = status_from_fields(&fields);
    Ok(NormalizedRecord {
        kind: kind.to_string(),
        identity,
        status,
        fields,
        payload_hashes,
        children,
    })
}

fn normalize_message_file(relative: &str, bytes: &[u8]) -> NormalizedRecord {
    let (header_bytes, body) = split_headers_body(bytes);
    let mut fields = parse_headers(header_bytes);
    fields.insert("body_sha256".to_string(), sha256_hex(body));
    let identity = fields
        .get("message-id")
        .cloned()
        .unwrap_or_else(|| relative.to_string());
    let payload_hashes = fields
        .get("body_sha256")
        .cloned()
        .into_iter()
        .collect();
    NormalizedRecord {
        kind: "message".to_string(),
        identity,
        status: EvidenceStatus::Present,
        fields,
        payload_hashes,
        children: Vec::new(),
    }
}

fn normalize_text_record(kind: &str, relative: &str, bytes: &[u8]) -> NormalizedRecord {
    let text = String::from_utf8_lossy(bytes);
    let mut fields = BTreeMap::new();
    if let Some(first) = text.lines().find(|line| !line.trim().is_empty()) {
        fields.insert("first_line".to_string(), first.trim().to_string());
    }
    fields.insert("payload_sha256".to_string(), sha256_hex(bytes));
    NormalizedRecord {
        kind: kind.to_string(),
        identity: relative.to_string(),
        status: EvidenceStatus::Present,
        fields,
        payload_hashes: vec![sha256_hex(bytes)],
        children: Vec::new(),
    }
}

fn normalize_mbox(relative: &str, bytes: &[u8]) -> Vec<NormalizedRecord> {
    let text = String::from_utf8_lossy(bytes);
    text.split("\nFrom ")
        .enumerate()
        .map(|(index, part)| {
            let part = if index == 0 {
                part
            } else {
                part.strip_prefix("From ").unwrap_or(part)
            };
            let mut record = normalize_message_file(
                &format!("{relative}#{index:06}"),
                part.as_bytes(),
            );
            record.kind = "message".to_string();
            record
        })
        .collect()
}

fn split_headers_body(bytes: &[u8]) -> (&[u8], &[u8]) {
    if let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
        (&bytes[..index], &bytes[index + 4..])
    } else if let Some(index) = bytes.windows(2).position(|window| window == b"\n\n") {
        (&bytes[..index], &bytes[index + 2..])
    } else {
        (bytes, &[])
    }
}

fn parse_headers(bytes: &[u8]) -> BTreeMap<String, String> {
    let text = String::from_utf8_lossy(bytes);
    let mut fields: BTreeMap<String, String> = BTreeMap::new();
    let mut current_name = None::<String>;
    for line in text.lines() {
        if line.chars().next().is_some_and(char::is_whitespace) {
            if let Some(name) = &current_name {
                if let Some(value) = fields.get_mut(name) {
                    value.push(' ');
                    value.push_str(line.trim());
                }
            }
            continue;
        }
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let name = name.trim().to_ascii_lowercase();
        let value = value.trim().to_string();
        current_name = Some(name.clone());
        fields
            .entry(name)
            .and_modify(|existing| {
                existing.push('\n');
                existing.push_str(&value);
            })
            .or_insert(value);
    }
    fields
}

fn identity_from_fields(fields: &BTreeMap<String, String>) -> Option<String> {
    [
        "source_id",
        "message_key",
        "folder_key",
        "body_key",
        "attachment_key",
        "source_node_id",
        "message-id",
        "path",
        "id",
    ]
    .iter()
    .find_map(|key| fields.get(*key).cloned())
}

fn status_from_fields(fields: &BTreeMap<String, String>) -> EvidenceStatus {
    let value = fields
        .get("status")
        .or_else(|| fields.get("body_status"))
        .map(String::as_str)
        .unwrap_or("present")
        .to_ascii_lowercase();
    if value.contains("unavailable") {
        EvidenceStatus::Unavailable
    } else if value.contains("unsupported") {
        EvidenceStatus::Unsupported
    } else if value.contains("ambiguous") {
        EvidenceStatus::Ambiguous
    } else if value.contains("malformed") {
        EvidenceStatus::Malformed
    } else if value.contains("corrupt") {
        EvidenceStatus::Corrupt
    } else if value.contains("failed") {
        EvidenceStatus::Failed
    } else if value.contains("filtered") {
        EvidenceStatus::Filtered
    } else if value.contains("empty") {
        EvidenceStatus::Empty
    } else {
        EvidenceStatus::Present
    }
}

fn is_volatile_field(key: &str) -> bool {
    matches!(
        key,
        "run_id" | "timestamp_utc" | "started_at" | "finished_at" | "duration_seconds"
    )
}

fn field_value(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        _ => canonical_json(value),
    }
}

fn canonical_value(value: &Value) -> String {
    canonical_json(value)
}

fn canonical_json(value: &Value) -> String {
    match value {
        Value::Object(object) => {
            let sorted = object
                .iter()
                .map(|(key, value)| (key.clone(), canonical_json(value)))
                .collect::<BTreeMap<_, _>>();
            serde_json::to_string(&sorted).unwrap_or_default()
        }
        Value::Array(array) => {
            let values = array.iter().map(canonical_json).collect::<Vec<_>>();
            serde_json::to_string(&values).unwrap_or_default()
        }
        _ => serde_json::to_string(value).unwrap_or_default(),
    }
}

fn find_files(root: &Path, predicate: impl Fn(&Path) -> bool) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    walk_files(root, &predicate, &mut files)?;
    Ok(files)
}

fn walk_files(
    current: &Path,
    predicate: &impl Fn(&Path) -> bool,
    files: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let mut entries = fs::read_dir(current)
        .map_err(|error| format!("normalization_read_failed:{}:{error}", current.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("normalization_entry_failed:{error}"))?;
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("normalization_metadata_failed:{}:{error}", path.display()))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("normalization_symlink_rejected: {}", path.display()));
        }
        if metadata.is_dir() {
            walk_files(&path, predicate, files)?;
        } else if metadata.is_file() && predicate(&path) {
            files.push(path);
        }
    }
    Ok(())
}

fn artifact_type(path: &str) -> String {
    Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .unwrap_or_else(|| "file".to_string())
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn normalizes_jsonl_records_and_volatile_fields() {
        let root = tempdir().expect("tempdir");
        let archive_path = root.path().join("canonical.tar");
        let file = File::create(&archive_path).expect("archive");
        let mut builder = tar::Builder::new(file);
        let body = br#"{"message_key":"m1","subject":"Hello","timestamp_utc":"now","status":"present"}
"#;
        let mut header = tar::Header::new_gnu();
        header.set_path("data/messages.jsonl").expect("path");
        header.set_size(body.len() as u64);
        header.set_cksum();
        builder.append(&header, &body[..]).expect("append");
        builder.finish().expect("finish");
        let result = normalize_pstd_archive(root.path(), "pstd", &NormalizationLimits::default())
            .expect("normalize");
        assert_eq!(result.records.len(), 1);
        assert_eq!(result.records[0].identity, "m1");
        assert!(!result.records[0].fields.contains_key("timestamp_utc"));
    }

    #[test]
    fn normalizes_headers_and_mbox_without_using_filenames_as_identity() {
        let root = tempdir().expect("tempdir");
        fs::write(
            root.path().join("0001.eml"),
            b"Message-ID: <m1>\r\nSubject: Hello\r\n\r\nBody",
        )
        .expect("eml");
        let result = normalize_readpst_directory(root.path(), "readpst", &NormalizationLimits::default())
            .expect("normalize");
        assert_eq!(result.records[0].identity, "<m1>");
        assert_eq!(result.records[0].fields["subject"], "Hello");
    }

    #[test]
    fn rejects_archive_path_traversal() {
        let root = tempdir().expect("tempdir");
        let archive_path = root.path().join("canonical.tar");
        let file = File::create(&archive_path).expect("archive");
        let mut builder = tar::Builder::new(file);
        let body = b"bad";
        let mut header = tar::Header::new_gnu();
        header.set_path("../escape").expect("path");
        header.set_size(body.len() as u64);
        header.set_cksum();
        builder.append(&header, &body[..]).expect("append");
        builder.finish().expect("finish");
        let error = normalize_pstd_archive(root.path(), "pstd", &NormalizationLimits::default())
            .expect_err("escape should fail");
        assert!(error.contains("path_escape"));
    }
}
