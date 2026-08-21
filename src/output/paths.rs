use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

use crate::error::{PstdError, PstdResult};

const MAX_SEGMENT_CHARS: usize = 120;

pub fn sanitize_segment(input: &str) -> String {
    let mut out = String::new();

    for ch in input.chars() {
        let safe = match ch {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            ch if ch.is_control() => '_',
            ch => ch,
        };
        out.push(safe);
    }

    let out = out.trim().trim_matches('.').to_string();
    let out = if out.is_empty() { "_".to_string() } else { out };
    out.chars().take(MAX_SEGMENT_CHARS).collect()
}

pub fn archive_path(parts: &[impl AsRef<str>]) -> PathBuf {
    let mut path = PathBuf::new();
    for part in parts {
        path.push(sanitize_segment(part.as_ref()));
    }
    path
}

pub fn archive_path_preserve_hidden(parts: &[impl AsRef<str>]) -> PathBuf {
    let mut path = PathBuf::new();
    for part in parts {
        let raw = part.as_ref();
        let safe = sanitize_segment(raw);
        if raw.starts_with('.') && raw != "." && raw != ".." {
            path.push(format!(".{safe}"));
        } else {
            path.push(safe);
        }
    }
    path
}

#[derive(Debug, Default)]
pub struct UniquePathTracker {
    seen: HashMap<String, usize>,
}

impl UniquePathTracker {
    pub fn unique_file_name(&mut self, original: &str) -> String {
        let safe = sanitize_segment(original);
        let count = self.seen.entry(safe.clone()).or_insert(0);
        *count += 1;

        if *count == 1 {
            return safe;
        }

        let path = Path::new(&safe);
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
        let ext = path.extension().and_then(|s| s.to_str());

        match ext {
            Some(ext) if !ext.is_empty() => format!("{stem}_{:04}.{ext}", *count),
            _ => format!("{stem}_{:04}", *count),
        }
    }
}

pub fn validate_archive_path(path: &Path) -> PstdResult<()> {
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
    {
        return Err(PstdError::OutputWrite(format!(
            "archive path is not confined to the output root: {}",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{archive_path, sanitize_segment, validate_archive_path, UniquePathTracker};

    #[test]
    fn sanitization_and_collision_policy_are_bounded() {
        assert_eq!(sanitize_segment("../secret"), "_secret");
        assert_eq!(archive_path(&["folder", "../secret"]), Path::new("folder/_secret"));

        let mut tracker = UniquePathTracker::default();
        assert_eq!(tracker.unique_file_name("same.eml"), "same.eml");
        assert_eq!(tracker.unique_file_name("same.eml"), "same_0002.eml");
    }

    #[test]
    fn archive_path_rejects_escape_components() {
        assert!(validate_archive_path(Path::new("../escape")).is_err());
        assert!(validate_archive_path(Path::new("/absolute")).is_err());
        assert!(validate_archive_path(Path::new("safe/file.txt")).is_ok());
    }
}
