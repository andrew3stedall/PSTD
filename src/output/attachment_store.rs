//! Retrieval of disk-materialized attachment bytes by stable attachment ID.

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::error::{PstdError, PstdResult};
use crate::output::metadata::AttachmentRecord;
use crate::pst::attachments::AttachmentPayload;

#[derive(Debug, Clone)]
pub struct RetrievedAttachment {
    pub record: AttachmentRecord,
    pub bytes: Vec<u8>,
}

/// Materialize validated attachment payloads beneath an extraction root and
/// return the sidecar manifest bytes. Metadata-only and integrity-failed
/// records remain in the manifest without a file path.
pub fn write_disk_attachments(
    extraction_root: impl AsRef<Path>,
    records: &[AttachmentRecord],
    payloads: &[AttachmentPayload],
) -> PstdResult<(Vec<u8>, usize)> {
    let root = extraction_root.as_ref();
    let index = crate::output::attachment_index::AttachmentIndex::new(records, payloads);
    let mut used_paths = std::collections::BTreeSet::new();
    let mut used_ids = std::collections::BTreeSet::new();
    // Validate all ambiguous mappings before writing any attachment bytes.
    for record in &index.records {
        let path = safe_relative_path(&record.archive_path)?;
        if !used_paths.insert(path) {
            return Err(PstdError::OutputWrite(format!(
                "duplicate attachment archive path: {}",
                record.archive_path
            )));
        }
        if !used_ids.insert(record.attachment_key.as_str()) {
            return Err(PstdError::OutputWrite(format!(
                "duplicate attachment ID in records: {}",
                record.attachment_key
            )));
        }
        index.payload(&record.attachment_key).map_err(|reason| {
            PstdError::OutputWrite(format!("{reason}: {}", record.attachment_key))
        })?;
    }
    let mut materialized_count = 0usize;
    let mut manifest = String::new();
    for record in &index.records {
        let relative_path = safe_relative_path(&record.archive_path)?;
        let payload = index.payload(&record.attachment_key).map_err(|reason| {
            PstdError::OutputWrite(format!("{reason}: {}", record.attachment_key))
        })?;
        let (materialized_path, status) = match payload {
            Some(payload) if payload_matches_record(payload, record) => {
                let path = root.join(&relative_path);
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&path, &payload.bytes)?;
                materialized_count += 1;
                (Some(record.archive_path.clone()), "attachment_file_emitted")
            }
            Some(_) => (None, "attachment_payload_integrity_failed"),
            None => (None, "attachment_payload_unavailable"),
        };

        let mut value = serde_json::to_value(&record)?;
        let Some(object) = value.as_object_mut() else {
            return Err(PstdError::OutputWrite(
                "attachment record did not serialize as an object".to_string(),
            ));
        };
        object.insert(
            "materialized_path".to_string(),
            materialized_path
                .map(serde_json::Value::String)
                .unwrap_or(serde_json::Value::Null),
        );
        object.insert(
            "materialization_status".to_string(),
            serde_json::Value::String(status.to_string()),
        );
        manifest.push_str(&serde_json::to_string(&value)?);
        manifest.push('\n');
    }
    fs::write(root.join("attachments.jsonl"), manifest.as_bytes())?;
    Ok((manifest.into_bytes(), materialized_count))
}

/// Retrieve an attachment from a disk-materialized extraction by its stable
/// `attachment_key`. The method accepts either an extraction root containing
/// `attachments.jsonl` or an unpacked canonical archive containing
/// `data/attachments.jsonl`.
pub fn retrieve_attachment_by_id(
    extraction_root: impl AsRef<Path>,
    attachment_id: &str,
) -> PstdResult<RetrievedAttachment> {
    if attachment_id.is_empty()
        || attachment_id.len() > 128
        || attachment_id.chars().any(|character| {
            !(character.is_ascii_alphanumeric() || character == '_' || character == '-')
        })
    {
        return Err(PstdError::InvalidConfig(
            "attachment ID contains unsupported characters".to_string(),
        ));
    }

    let root = extraction_root.as_ref();
    let manifest_path = [
        root.join("attachments.jsonl"),
        root.join("data/attachments.jsonl"),
    ]
    .into_iter()
    .find(|path| path.is_file())
    .ok_or_else(|| {
        PstdError::SourceOpen(format!(
            "attachment manifest not found beneath {}",
            root.display()
        ))
    })?;
    let mut manifest = BufReader::new(fs::File::open(&manifest_path)?);
    let mut found = None;
    let mut line = String::new();
    loop {
        line.clear();
        if manifest.read_line(&mut line)? == 0 {
            break;
        }
        if line.trim().is_empty() {
            continue;
        }
        let record = serde_json::from_str::<AttachmentRecord>(&line).map_err(PstdError::Json)?;
        if record.attachment_key == attachment_id {
            if found.is_some() {
                return Err(PstdError::OutputWrite(format!(
                    "attachment manifest contains duplicate attachment ID: {attachment_id}"
                )));
            }
            found = Some(record);
        }
    }
    let record = found.ok_or_else(|| {
        PstdError::SourceOpen(format!(
            "attachment ID not found in manifest: {attachment_id}"
        ))
    })?;
    let relative_path = safe_relative_path(&record.archive_path)?;
    let path = root.join(relative_path);
    let bytes = fs::read(&path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            PstdError::SourceOpen(format!(
                "attachment {} is not disk-materialized at {}",
                attachment_id,
                path.display()
            ))
        } else {
            PstdError::Io(error)
        }
    })?;
    if bytes.len() as u64 != record.size_bytes || sha256_hex(&bytes) != record.sha256 {
        return Err(PstdError::OutputWrite(format!(
            "attachment {} failed size/hash validation",
            attachment_id
        )));
    }
    Ok(RetrievedAttachment { record, bytes })
}

fn safe_relative_path(path: &str) -> PstdResult<PathBuf> {
    let path = Path::new(path);
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::CurDir
                    | Component::ParentDir
                    | Component::RootDir
                    | Component::Prefix(_)
            )
        })
    {
        return Err(PstdError::OutputWrite(format!(
            "attachment archive path is not confined to extraction root: {}",
            path.display()
        )));
    }
    Ok(path.to_path_buf())
}

fn payload_matches_record(payload: &AttachmentPayload, record: &AttachmentRecord) -> bool {
    payload.record.message_key == record.message_key
        && payload.record.attachment_key == record.attachment_key
        && payload.record.archive_path == record.archive_path
        && payload.bytes.len() as u64 == record.size_bytes
        && sha256_hex(&payload.bytes) == record.sha256
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{retrieve_attachment_by_id, write_disk_attachments};
    use crate::pst::attachments::{attachment_payload, AttachmentMetadata};

    #[test]
    fn retrieves_disk_materialized_attachment_by_stable_id() {
        let directory = tempdir().unwrap();
        let payload = attachment_payload(
            "msg_1",
            0,
            AttachmentMetadata {
                filename_original: Some("report.pdf".to_string()),
                ..Default::default()
            },
            b"pdf bytes".to_vec(),
        );
        let path = directory.path().join(&payload.record.archive_path);
        let (_, materialized_count) = write_disk_attachments(
            directory.path(),
            std::slice::from_ref(&payload.record),
            std::slice::from_ref(&payload),
        )
        .unwrap();
        assert_eq!(materialized_count, 1);
        assert!(path.is_file());

        let retrieved =
            retrieve_attachment_by_id(directory.path(), &payload.record.attachment_key).unwrap();
        assert_eq!(retrieved.record.message_key, "msg_1");
        assert_eq!(retrieved.bytes, b"pdf bytes");
    }
}
