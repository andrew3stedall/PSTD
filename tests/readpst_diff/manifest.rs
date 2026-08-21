use std::path::{Component, Path};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const READPST_SOURCE_REVISION: &str =
    "cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParityStatus {
    Implemented,
    Partial,
    Gap,
    UnsupportedByReadpst,
    Filtered,
    Unavailable,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceLevel {
    #[serde(rename = "E0")]
    E0,
    #[serde(rename = "E1")]
    E1,
    #[serde(rename = "E2")]
    E2,
    #[serde(rename = "E3")]
    E3,
    #[serde(rename = "E4")]
    E4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputFamily {
    Ansi32,
    Unicode64,
    Ost2013,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CryptMethod {
    None,
    Compressible,
    Strong,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixtureAdmission {
    Approved,
    Candidate,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStatus {
    Present,
    Empty,
    Skipped,
    Filtered,
    Unavailable,
    Unsupported,
    Ambiguous,
    Malformed,
    Corrupt,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceProvenance {
    pub source_url: String,
    pub source_revision: String,
    pub source_path: String,
    pub license_basis: String,
}

impl SourceProvenance {
    pub fn validate(&self) -> Result<(), String> {
        for (field, value) in [
            ("source_url", &self.source_url),
            ("source_revision", &self.source_revision),
            ("source_path", &self.source_path),
            ("license_basis", &self.license_basis),
        ] {
            if value.trim().is_empty() {
                return Err(format!("{field} must not be empty"));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureManifest {
    pub fixture_id: String,
    pub provenance: SourceProvenance,
    pub local_path: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub input_family: InputFamily,
    pub crypt_method: CryptMethod,
    pub expected_category: String,
    pub expected_status: EvidenceStatus,
    pub admission: FixtureAdmission,
}

impl FixtureManifest {
    pub fn approved_unicode_tika() -> Self {
        Self {
            fixture_id: "unicode_tika_testpst".to_string(),
            provenance: SourceProvenance {
                source_url: "https://github.com/apache/tika/blob/63e22d08ef249cc73a6d02da7bc199fc3623a607/tika-app/src/test/resources/test-data/testPST.pst".to_string(),
                source_revision: "63e22d08ef249cc73a6d02da7bc199fc3623a607".to_string(),
                source_path: "tika-app/src/test/resources/test-data/testPST.pst".to_string(),
                license_basis: "Apache License 2.0; approved redistributable test fixture".to_string(),
            },
            local_path: "tests/fixtures/upstream/tika-testPST.pst".to_string(),
            sha256: "f2a6b1d2cad00f574e3d1c1211c4b1c854d6526caea77213adc3da92b7813ae3".to_string(),
            size_bytes: 2_302_976,
            input_family: InputFamily::Unicode64,
            crypt_method: CryptMethod::None,
            expected_category: "unicode_message_attachment_embedded".to_string(),
            expected_status: EvidenceStatus::Present,
            admission: FixtureAdmission::Approved,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.fixture_id.trim().is_empty() {
            return Err("fixture_id must not be empty".to_string());
        }
        self.provenance.validate()?;
        validate_relative_path("local_path", &self.local_path)?;
        validate_sha256("sha256", &self.sha256)?;
        if self.size_bytes == 0 {
            return Err("size_bytes must be greater than zero".to_string());
        }
        if self.expected_category.trim().is_empty() {
            return Err("expected_category must not be empty".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecution {
    pub tool: String,
    pub version: String,
    pub command: Vec<String>,
    pub exit_status: Option<i32>,
    pub stdout_sha256: Option<String>,
    pub stderr_sha256: Option<String>,
    pub output_root: String,
    pub status: EvidenceStatus,
}

impl ToolExecution {
    pub fn validate(&self) -> Result<(), String> {
        if self.tool.trim().is_empty() {
            return Err("tool must not be empty".to_string());
        }
        if self.version.trim().is_empty() {
            return Err("tool version must not be empty".to_string());
        }
        if self.command.is_empty() || self.command[0].trim().is_empty() {
            return Err("command must contain an executable".to_string());
        }
        validate_relative_path("output_root", &self.output_root)?;
        if let Some(hash) = &self.stdout_sha256 {
            validate_sha256("stdout_sha256", hash)?;
        }
        if let Some(hash) = &self.stderr_sha256 {
            validate_sha256("stderr_sha256", hash)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeRecord {
    pub case_id: String,
    pub scope: String,
    pub expected: EvidenceStatus,
    pub observed: EvidenceStatus,
    pub reason_code: String,
    pub detail: Option<String>,
}

impl OutcomeRecord {
    pub fn validate(&self) -> Result<(), String> {
        for (field, value) in [
            ("case_id", &self.case_id),
            ("scope", &self.scope),
            ("reason_code", &self.reason_code),
        ] {
            if value.trim().is_empty() {
                return Err(format!("{field} must not be empty"));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonRun {
    pub comparison_id: String,
    pub fixture_id: String,
    pub fixture_sha256: String,
    pub readpst_source_revision: String,
    pub input_family: InputFamily,
    pub crypt_method: CryptMethod,
    pub charset_policy: String,
    pub output_profile: String,
    pub worker_count: u32,
    pub evidence_level: EvidenceLevel,
    pub parity_status: ParityStatus,
    pub readpst: ToolExecution,
    pub pstd: ToolExecution,
    pub outcomes: Vec<OutcomeRecord>,
}

impl ComparisonRun {
    pub fn validate(&self) -> Result<(), String> {
        if self.comparison_id.trim().is_empty() {
            return Err("comparison_id must not be empty".to_string());
        }
        if self.fixture_id.trim().is_empty() {
            return Err("fixture_id must not be empty".to_string());
        }
        validate_sha256("fixture_sha256", &self.fixture_sha256)?;
        if self.readpst_source_revision != READPST_SOURCE_REVISION {
            return Err(format!(
                "readpst_source_revision must equal pinned revision {READPST_SOURCE_REVISION}"
            ));
        }
        if self.charset_policy.trim().is_empty() {
            return Err("charset_policy must not be empty".to_string());
        }
        if self.output_profile.trim().is_empty() {
            return Err("output_profile must not be empty".to_string());
        }
        if self.worker_count == 0 {
            return Err("worker_count must be at least one".to_string());
        }
        self.readpst.validate()?;
        self.pstd.validate()?;
        if self.outcomes.is_empty() {
            return Err("comparison must contain at least one outcome record".to_string());
        }
        for outcome in &self.outcomes {
            outcome.validate()?;
        }
        if self.parity_status == ParityStatus::Implemented
            && self.evidence_level != EvidenceLevel::E4
        {
            return Err("implemented parity requires E4 evidence".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactDigest {
    pub artifact_type: String,
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
}

impl ArtifactDigest {
    pub fn validate(&self) -> Result<(), String> {
        if self.artifact_type.trim().is_empty() {
            return Err("artifact_type must not be empty".to_string());
        }
        validate_relative_path("artifact path", &self.path)?;
        validate_sha256("artifact sha256", &self.sha256)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryCounts {
    pub folders: u64,
    pub items: u64,
    pub messages: u64,
    pub bodies: u64,
    pub recipients: u64,
    pub attachments: u64,
    pub payload_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceReport {
    pub report_id: String,
    pub fixture_id: String,
    pub fixture_sha256: String,
    pub evidence_level: EvidenceLevel,
    pub parity_status: ParityStatus,
    pub inventory: Option<InventoryCounts>,
    pub outcomes: Vec<OutcomeRecord>,
    pub artifacts: Vec<ArtifactDigest>,
    pub deterministic_repeat: bool,
    pub notes: Vec<String>,
}

impl EvidenceReport {
    pub fn validate(&self) -> Result<(), String> {
        if self.report_id.trim().is_empty() {
            return Err("report_id must not be empty".to_string());
        }
        if self.fixture_id.trim().is_empty() {
            return Err("fixture_id must not be empty".to_string());
        }
        validate_sha256("fixture_sha256", &self.fixture_sha256)?;
        if self.outcomes.is_empty() {
            return Err("evidence report must contain at least one outcome".to_string());
        }
        for outcome in &self.outcomes {
            outcome.validate()?;
        }
        for artifact in &self.artifacts {
            artifact.validate()?;
        }
        if self.parity_status == ParityStatus::Implemented
            && self.evidence_level != EvidenceLevel::E4
        {
            return Err("implemented parity requires E4 evidence".to_string());
        }
        Ok(())
    }
}

pub fn approved_unicode_baseline_report() -> EvidenceReport {
    let fixture = FixtureManifest::approved_unicode_tika();
    EvidenceReport {
        report_id: "evidence_unicode_tika_baseline".to_string(),
        fixture_id: fixture.fixture_id,
        fixture_sha256: fixture.sha256,
        evidence_level: EvidenceLevel::E2,
        parity_status: ParityStatus::Partial,
        inventory: Some(InventoryCounts {
            folders: 8,
            items: 8,
            messages: 8,
            bodies: 10,
            recipients: 9,
            attachments: 2,
            payload_bytes: 12_315,
        }),
        outcomes: vec![
            OutcomeRecord {
                case_id: "unicode_tika_positive".to_string(),
                scope: "fixture".to_string(),
                expected: EvidenceStatus::Present,
                observed: EvidenceStatus::Present,
                reason_code: "approved_unicode_baseline".to_string(),
                detail: Some("Existing deterministic PSTD canonical extraction baseline.".to_string()),
            },
            OutcomeRecord {
                case_id: "unicode_tika_embedded_child".to_string(),
                scope: "embedded_message".to_string(),
                expected: EvidenceStatus::Present,
                observed: EvidenceStatus::Present,
                reason_code: "method_5_child_baseline".to_string(),
                detail: Some("One validated method-5 child layout is represented.".to_string()),
            },
        ],
        artifacts: Vec::new(),
        deterministic_repeat: true,
        notes: vec![
            "This is E2 fixture evidence, not a general readpst parity claim.".to_string(),
            "The fixture remains subject to the pinned provenance and SHA-256 admission record."
                .to_string(),
        ],
    }
}

pub fn stable_json<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    serde_json::to_vec(value).map_err(|error| format!("stable JSON serialization failed: {error}"))
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn validate_sha256(field: &str, value: &str) -> Result<(), String> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{field} must be a 64-character hexadecimal SHA-256"));
    }
    Ok(())
}

fn validate_relative_path(field: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    let path = Path::new(value);
    if path.is_absolute() {
        return Err(format!("{field} must be relative"));
    }
    if path.components().any(|component| {
        matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_))
    }) {
        return Err(format!("{field} must not contain traversal components"));
    }
    Ok(())
}
