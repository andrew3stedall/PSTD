use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
