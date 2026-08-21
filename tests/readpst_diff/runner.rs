use super::manifest::{
    sha256_hex, ArtifactDigest, EvidenceStatus, FixtureManifest, ToolExecution,
};
use serde::Serialize;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct RunLimits {
    pub max_runtime: Duration,
    pub max_stdout_bytes: usize,
    pub max_stderr_bytes: usize,
    pub max_files: usize,
    pub max_file_bytes: u64,
    pub max_total_bytes: u64,
}

impl Default for RunLimits {
    fn default() -> Self {
        Self {
            max_runtime: Duration::from_secs(30),
            max_stdout_bytes: 1_048_576,
            max_stderr_bytes: 1_048_576,
            max_files: 10_000,
            max_file_bytes: 64 * 1024 * 1024,
            max_total_bytes: 256 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub tool: String,
    pub version: String,
    pub command: Vec<String>,
    pub sandbox_root: PathBuf,
    pub output_root: PathBuf,
}

impl CommandSpec {
    pub fn new(
        tool: impl Into<String>,
        version: impl Into<String>,
        command: Vec<String>,
        sandbox_root: impl Into<PathBuf>,
        output_root: impl Into<PathBuf>,
    ) -> Self {
        Self {
            tool: tool.into(),
            version: version.into(),
            command,
            sandbox_root: sandbox_root.into(),
            output_root: output_root.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RunResult {
    pub execution: ToolExecution,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub artifacts: Vec<ArtifactDigest>,
    pub escaped_paths: Vec<String>,
    pub timed_out: bool,
    pub output_limited: bool,
}

#[derive(Debug)]
struct CapturedStream {
    bytes: Vec<u8>,
    limited: bool,
}

pub fn validate_fixture_on_disk(
    fixture: &FixtureManifest,
    repository_root: &Path,
) -> Result<PathBuf, String> {
    fixture.validate()?;
    let root = fs::canonicalize(repository_root)
        .map_err(|error| format!("repository_root_unavailable: {error}"))?;
    let candidate = repository_root.join(&fixture.local_path);
    let resolved = fs::canonicalize(&candidate)
        .map_err(|error| format!("fixture_unavailable:{}:{error}", fixture.local_path))?;
    if !resolved.starts_with(&root) {
        return Err(format!(
            "fixture_path_escape: {}",
            fixture.local_path
        ));
    }
    let metadata = fs::metadata(&resolved)
        .map_err(|error| format!("fixture_metadata_unavailable:{error}"))?;
    if !metadata.is_file() {
        return Err(format!("fixture_is_not_file: {}", fixture.local_path));
    }
    if metadata.len() != fixture.size_bytes {
        return Err(format!(
            "fixture_size_mismatch: expected {} observed {}",
            fixture.size_bytes,
            metadata.len()
        ));
    }
    let bytes = fs::read(&resolved).map_err(|error| format!("fixture_read_failed:{error}"))?;
    let observed = sha256_hex(&bytes);
    if observed != fixture.sha256 {
        return Err(format!(
            "fixture_sha256_mismatch: expected {} observed {}",
            fixture.sha256, observed
        ));
    }
    Ok(resolved)
}

pub fn run_isolated(spec: &CommandSpec, limits: &RunLimits) -> Result<RunResult, String> {
    if spec.command.is_empty() || spec.command[0].trim().is_empty() {
        return Err("command must contain an executable".to_string());
    }
    if spec.tool.trim().is_empty() || spec.version.trim().is_empty() {
        return Err("tool and version must not be empty".to_string());
    }
    fs::create_dir_all(&spec.sandbox_root)
        .map_err(|error| format!("sandbox_create_failed:{error}"))?;
    fs::create_dir_all(&spec.output_root)
        .map_err(|error| format!("output_root_create_failed:{error}"))?;
    let sandbox = fs::canonicalize(&spec.sandbox_root)
        .map_err(|error| format!("sandbox_unavailable:{error}"))?;
    let output_root = fs::canonicalize(&spec.output_root)
        .map_err(|error| format!("output_root_unavailable:{error}"))?;
    let output_relative = output_root
        .strip_prefix(&sandbox)
        .map_err(|_| "output_root_must_be_inside_sandbox".to_string())?;
    if output_relative.as_os_str().is_empty() {
        return Err("output_root_must_not_equal_sandbox".to_string());
    }
    if !sandbox_has_only_output_root(&sandbox, &output_root)? {
        return Err("sandbox_root_must_be_dedicated_and_empty".to_string());
    }

    let mut command = Command::new(&spec.command[0]);
    command
        .args(spec.command.iter().skip(1))
        .current_dir(&output_root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|error| format!("process_spawn_failed:{error}"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "stdout_pipe_unavailable".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "stderr_pipe_unavailable".to_string())?;
    let limit_hit = Arc::new(AtomicBool::new(false));
    let stdout_limit = Arc::clone(&limit_hit);
    let stderr_limit = Arc::clone(&limit_hit);
    let stdout_thread = thread::spawn(move || capture_stream(stdout, limits.max_stdout_bytes, stdout_limit));
    let stderr_thread = thread::spawn(move || capture_stream(stderr, limits.max_stderr_bytes, stderr_limit));

    let started = Instant::now();
    let mut timed_out = false;
    let mut output_limited = false;
    let exit_status = loop {
        if limit_hit.load(Ordering::Relaxed) {
            output_limited = true;
            let _ = child.kill();
            let status = child
                .wait()
                .map_err(|error| format!("process_wait_failed:{error}"))?;
            break status.code();
        }
        if started.elapsed() >= limits.max_runtime {
            timed_out = true;
            let _ = child.kill();
            let status = child
                .wait()
                .map_err(|error| format!("process_wait_failed:{error}"))?;
            break status.code();
        }
        match child.try_wait() {
            Ok(Some(status)) => break status.code(),
            Ok(None) => thread::sleep(Duration::from_millis(5)),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("process_poll_failed:{error}"));
            }
        }
    };

    let stdout = stdout_thread
        .join()
        .map_err(|_| "stdout_capture_thread_failed".to_string())?;
    let stderr = stderr_thread
        .join()
        .map_err(|_| "stderr_capture_thread_failed".to_string())?;
    output_limited |= stdout.limited || stderr.limited;
    let escaped_paths = find_escaped_paths(&sandbox, &output_root)?;
    let artifacts = collect_artifacts(&output_root, limits)?;

    let status = if exit_status == Some(0)
        && !timed_out
        && !output_limited
        && escaped_paths.is_empty()
    {
        EvidenceStatus::Present
    } else {
        EvidenceStatus::Failed
    };
    let execution_output_root = path_string(output_relative);
    let stable_command = stable_command(&spec.command, &sandbox, &output_root);
    let execution = ToolExecution {
        tool: spec.tool.clone(),
        version: spec.version.clone(),
        command: stable_command,
        exit_status,
        stdout_sha256: Some(sha256_hex(&stdout.bytes)),
        stderr_sha256: Some(sha256_hex(&stderr.bytes)),
        output_root: execution_output_root,
        status,
    };
    execution.validate()?;

    Ok(RunResult {
        execution,
        stdout: stdout.bytes,
        stderr: stderr.bytes,
        artifacts,
        escaped_paths,
        timed_out,
        output_limited,
    })
}

fn stable_command(command: &[String], sandbox: &Path, output_root: &Path) -> Vec<String> {
    let sandbox = path_string(sandbox);
    let output_root = path_string(output_root);
    command
        .iter()
        .map(|argument| {
            argument
                .replace(&sandbox, "{sandbox}")
                .replace(&output_root, "{output_root}")
        })
        .collect()
}

fn capture_stream<R: Read>(
    mut reader: R,
    limit: usize,
    limit_hit: Arc<AtomicBool>,
) -> CapturedStream {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 8192];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => {
                let remaining = limit.saturating_sub(bytes.len());
                if read > remaining {
                    bytes.extend_from_slice(&buffer[..remaining]);
                    limit_hit.store(true, Ordering::Relaxed);
                    return CapturedStream {
                        bytes,
                        limited: true,
                    };
                }
                bytes.extend_from_slice(&buffer[..read]);
            }
            Err(_) => {
                limit_hit.store(true, Ordering::Relaxed);
                return CapturedStream {
                    bytes,
                    limited: true,
                };
            }
        }
    }
    CapturedStream {
        bytes,
        limited: false,
    }
}

fn sandbox_has_only_output_root(sandbox: &Path, output_root: &Path) -> Result<bool, String> {
    let entries = fs::read_dir(sandbox).map_err(|error| format!("sandbox_read_failed:{error}"))?;
    for entry in entries {
        let path = entry
            .map_err(|error| format!("sandbox_entry_failed:{error}"))?
            .path();
        if path != output_root {
            return Ok(false);
        }
    }
    Ok(true)
}

fn find_escaped_paths(sandbox: &Path, output_root: &Path) -> Result<Vec<String>, String> {
    let mut escaped = Vec::new();
    let entries = fs::read_dir(sandbox).map_err(|error| format!("sandbox_read_failed:{error}"))?;
    for entry in entries {
        let path = entry
            .map_err(|error| format!("sandbox_entry_failed:{error}"))?
            .path();
        if path != output_root {
            escaped.push(path_string(
                path.strip_prefix(sandbox)
                    .map_err(|_| "sandbox_relative_path_failed".to_string())?,
            ));
        }
    }
    escaped.sort();
    Ok(escaped)
}

fn collect_artifacts(root: &Path, limits: &RunLimits) -> Result<Vec<ArtifactDigest>, String> {
    let mut files = Vec::new();
    collect_files(root, root, limits, &mut files)?;
    files.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(files
        .into_iter()
        .map(|(path, size, hash)| ArtifactDigest {
            artifact_type: artifact_type(&path),
            path,
            sha256: hash,
            size_bytes: size,
        })
        .collect())
}

fn collect_files(
    root: &Path,
    current: &Path,
    limits: &RunLimits,
    files: &mut Vec<(String, u64, String)>,
) -> Result<u64, String> {
    let mut entries = fs::read_dir(current)
        .map_err(|error| format!("output_read_failed:{}:{error}", current.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("output_entry_failed:{error}"))?;
    entries.sort_by_key(|entry| entry.path());
    let mut total = 0_u64;
    for entry in entries {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("output_metadata_failed:{}:{error}", path.display()))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("output_symlink_rejected: {}", path.display()));
        }
        if metadata.is_dir() {
            total = total
                .checked_add(collect_files(root, &path, limits, files)?)
                .ok_or_else(|| "output_total_size_overflow".to_string())?;
            continue;
        }
        if !metadata.is_file() {
            return Err(format!("output_non_file_rejected: {}", path.display()));
        }
        if files.len() >= limits.max_files {
            return Err(format!("output_file_count_exceeded: {}", limits.max_files));
        }
        if metadata.len() > limits.max_file_bytes {
            return Err(format!(
                "output_file_size_exceeded: {}",
                path.display()
            ));
        }
        total = total
            .checked_add(metadata.len())
            .ok_or_else(|| "output_total_size_overflow".to_string())?;
        if total > limits.max_total_bytes {
            return Err(format!(
                "output_total_size_exceeded: {}",
                limits.max_total_bytes
            ));
        }
        let bytes = fs::read(&path).map_err(|error| format!("output_read_failed:{error}"))?;
        let relative = path_string(
            path.strip_prefix(root)
                .map_err(|_| "output_relative_path_failed".to_string())?,
        );
        files.push((relative, metadata.len(), sha256_hex(&bytes)));
    }
    Ok(total)
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

    fn spec(root: &Path, command: Vec<String>) -> CommandSpec {
        let output = root.join("output");
        CommandSpec::new("stub-readpst", "test-1", command, root, output)
    }

    #[test]
    fn captures_success_and_artifacts_with_stable_hashes() {
        let root = tempdir().expect("tempdir");
        let result = run_isolated(
            &spec(
                root.path(),
                vec![
                    "sh".to_string(),
                    "-c".to_string(),
                    "printf 'stdout'; printf 'stderr' >&2; printf 'payload' > message.eml",
                ],
            ),
            &RunLimits::default(),
        )
        .expect("run should succeed");
        assert_eq!(result.execution.status, EvidenceStatus::Present);
        assert_eq!(result.execution.exit_status, Some(0));
        assert_eq!(result.stdout, b"stdout");
        assert_eq!(result.stderr, b"stderr");
        assert_eq!(result.artifacts.len(), 1);
        assert_eq!(result.artifacts[0].path, "message.eml");
        assert!(result.escaped_paths.is_empty());
    }

    #[test]
    fn rejects_output_limits_timeouts_and_path_escape() {
        let root = tempdir().expect("tempdir");
        let limits = RunLimits {
            max_runtime: Duration::from_millis(100),
            max_stdout_bytes: 4,
            ..RunLimits::default()
        };
        let result = run_isolated(
            &spec(
                root.path(),
                vec![
                    "sh".to_string(),
                    "-c".to_string(),
                    "printf 'this output is too long'; touch ../escape",
                ],
            ),
            &limits,
        )
        .expect("bounded run should return evidence");
        assert_eq!(result.execution.status, EvidenceStatus::Failed);
        assert!(result.output_limited || !result.escaped_paths.is_empty());
    }

    #[test]
    fn timeout_is_explicit_and_child_is_terminated() {
        let root = tempdir().expect("tempdir");
        let limits = RunLimits {
            max_runtime: Duration::from_millis(50),
            ..RunLimits::default()
        };
        let result = run_isolated(
            &spec(
                root.path(),
                vec!["sh".to_string(), "-c".to_string(), "sleep 1"],
            ),
            &limits,
        )
        .expect("timeout should return evidence");
        assert_eq!(result.execution.status, EvidenceStatus::Failed);
        assert!(result.timed_out);
    }
}
