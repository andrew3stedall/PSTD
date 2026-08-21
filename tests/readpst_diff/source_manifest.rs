use serde::Serialize;
use std::collections::BTreeSet;

use super::manifest::READPST_SOURCE_REVISION;

pub const UPSTREAM_REPOSITORY: &str = "pst-format/libpst";
pub const UPSTREAM_BLOB_URL: &str = "https://github.com/pst-format/libpst/blob/";
pub const LICENSE_BOUNDARY_NOTE: &str =
    "The upstream GPL source is an oracle and review source only; no implementation is copied into PSTD.";

#[derive(Debug, Clone, Serialize)]
pub struct SourceEntry {
    pub path: &'static str,
    pub line_ref: &'static str,
    pub symbols: &'static [&'static str],
    pub behavior: &'static str,
    pub work_units: &'static [&'static str],
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkUnitAnchor {
    pub key: &'static str,
    pub issue: u16,
    pub path: &'static str,
    pub symbol: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegressionProfile {
    pub name: &'static str,
    pub command_shape: &'static str,
    pub evidence_scope: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct SourceManifest {
    pub repository: &'static str,
    pub revision: &'static str,
    pub license_boundary: &'static str,
    pub entries: &'static [SourceEntry],
    pub work_unit_anchors: &'static [WorkUnitAnchor],
    pub regression_profiles: &'static [RegressionProfile],
    pub out_of_scope_utilities: &'static [&'static str],
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SourceDriftReport {
    pub expected_revision: String,
    pub observed_revision: String,
    pub status: String,
    pub missing_paths: Vec<String>,
    pub duplicate_paths: Vec<String>,
    pub unresolved_work_units: Vec<String>,
    pub detail: String,
}

const REQUIRED_DIRECT_PATHS: &[&str] = &[
    "src/readpst.c",
    "src/libpst.c",
    "src/libpst.h",
    "src/msg.cpp",
    "src/msg.h",
    "src/lzfu.c",
    "src/lzfu.h",
    "src/vbuf.c",
    "src/vbuf.h",
    "src/timeconv.c",
    "src/timeconv.h",
    "src/libstrfunc.c",
    "src/libstrfunc.h",
    "src/debug.c",
    "src/define.h",
    "src/common.h",
    "src/XGetopt.c",
    "src/XGetopt.h",
    "src/Makefile.am",
    "regression/regression-tests.bash",
    "NEWS",
    "ChangeLog",
];

const READPST_SYMBOLS: &[&str] = &[
    "main",
    "process",
    "try_fork",
    "create_enter_dir",
    "close_enter_dir",
    "mk_*",
    "mk_separate_dir",
    "mk_separate_file",
    "mk_kmail_dir",
    "mk_kmail_file",
    "mk_thunderbird_dir",
    "acceptable_ext",
    "write_separate_attachment",
    "write_embedded_message",
    "write_inline_attachment",
    "valid_headers",
    "write_body_part",
    "write_normal_email",
    "write_schedule_part_data",
    "write_vcard",
    "write_extra_categories",
    "write_journal",
    "write_appointment",
];

const LIBPST_C_SYMBOLS: &[&str] = &[
    "pst_open",
    "pst_reopen",
    "pst_load_index",
    "pst_load_extended_attributes",
    "pst_getTopOfFolders",
    "pst_parse_item",
    "pst_process",
    "pst_attach_to_file",
    "pst_attach_to_file_base64",
    "pst_default_charset",
    "pst_rfc2231",
    "pst_rfc2047",
    "pst_convert_recurrence",
];

const LIBPST_H_SYMBOLS: &[&str] = &[
    "item type constants",
    "attachment method constants",
    "encryption constants",
    "message flags",
    "email/contact/appointment/journal structures",
    "store and attachment structures",
];

const MSG_CPP_SYMBOLS: &[&str] = &[
    "write_msg_email",
    "property helpers",
    "recipient loop",
    "attachment loop",
    "NameID stream",
];

const MSG_H_SYMBOLS: &[&str] = &["MSG property and storage declarations"];
const LZFU_C_SYMBOLS: &[&str] = &["pst_lzfu_decompress", "dictionary/header/flag loop"];
const LZFU_H_SYMBOLS: &[&str] = &["LZFU declarations"];
const VBUF_C_SYMBOLS: &[&str] = &[
    "buffer growth",
    "pst_unicode_init",
    "pst_unicode_close",
    "UTF-16 to UTF-8 conversion",
    "UTF-8 to target charset conversion",
];
const VBUF_H_SYMBOLS: &[&str] = &["charset buffer declarations"];
const TIMECONV_C_SYMBOLS: &[&str] = &[
    "FILETIME to Unix",
    "FILETIME to ASCII",
    "FILETIME to string",
    "FILETIME to UTC tm",
];
const TIMECONV_H_SYMBOLS: &[&str] = &["FILETIME conversion declarations"];
const LIBSTRFUNC_C_SYMBOLS: &[&str] = &["pst_base64_encode*", "76-column line handling"];
const LIBSTRFUNC_H_SYMBOLS: &[&str] = &["base64 declarations"];
const DEBUG_C_SYMBOLS: &[&str] = &["debug macros", "allocation/error helpers"];
const DEFINE_H_SYMBOLS: &[&str] = &["portability definitions", "conditional feature definitions"];
const COMMON_H_SYMBOLS: &[&str] = &["common allocation/error declarations"];
const XGETOPT_C_SYMBOLS: &[&str] = &["fallback getopt implementation"];
const XGETOPT_H_SYMBOLS: &[&str] = &["fallback getopt declarations"];
const MAKEFILE_SYMBOLS: &[&str] = &["common_source", "readpst_SOURCES", "readpst_LDADD"];
const REGRESSION_SYMBOLS: &[&str] = &[
    "consistency",
    "dodii",
    "doldif",
    "dopst",
    "profile commands",
    "fixture list",
    "valgrind/size gates",
];
const NEWS_SYMBOLS: &[&str] = &[
    "release notes",
    "encryption type 2",
    "OST 2013",
    "Content-ID",
    "RFC 2047/2231",
    "reports",
    "embedded messages",
    "recurrence",
    "mixed types",
];
const CHANGELOG_SYMBOLS: &[&str] = &["historical compatibility fixes", "release behavior notes"];

const DIRECT_ENTRIES: &[SourceEntry] = &[
    SourceEntry {
        path: "src/readpst.c",
        line_ref: "#L243-L2490",
        symbols: READPST_SYMBOLS,
        behavior: "CLI parsing, traversal, routing, output layouts, MIME, attachments, contacts, journals, appointments, counts, and diagnostics.",
        work_units: &[
            "RP-M0-01", "RP-M1-02", "RP-M1-03", "RP-M2-01", "RP-M2-02",
            "RP-M2-03", "RP-M2-04", "RP-M3-01", "RP-M3-02", "RP-M3-03",
            "RP-M4-01", "RP-M4-02", "RP-M4-03", "RP-M5-01", "RP-M5-02",
            "RP-M5-03", "RP-M6-03",
        ],
    },
    SourceEntry {
        path: "src/libpst.c",
        line_ref: "#L315-L4565",
        symbols: LIBPST_C_SYMBOLS,
        behavior: "Opening, family/encryption handling, indexes, properties, item projection, ID2 references, charset conversion, and recurrence.",
        work_units: &["RP-M1-01", "RP-M1-02", "RP-M1-04", "RP-M2-02", "RP-M2-03", "RP-M4-02", "RP-M6-01", "RP-M6-02"],
    },
    SourceEntry {
        path: "src/libpst.h",
        line_ref: "",
        symbols: LIBPST_H_SYMBOLS,
        behavior: "Semantic model consumed by readpst: item classes, methods, flags, and typed item/storage structures.",
        work_units: &["RP-M1-02", "RP-M1-03", "RP-M2-01", "RP-M2-03", "RP-M4-01", "RP-M4-02"],
    },
    SourceEntry {
        path: "src/msg.cpp",
        line_ref: "#L20-L434",
        symbols: MSG_CPP_SYMBOLS,
        behavior: "OLE MSG compound-document writer, MAPI property streams, recipients, attachments, and embedded-message limitations.",
        work_units: &["RP-M2-01", "RP-M3-01", "RP-M5-04"],
    },
    SourceEntry {
        path: "src/msg.h",
        line_ref: "",
        symbols: MSG_H_SYMBOLS,
        behavior: "MSG writer declarations and property/storage boundary.",
        work_units: &["RP-M5-04"],
    },
    SourceEntry {
        path: "src/lzfu.c",
        line_ref: "#L38-L120",
        symbols: LZFU_C_SYMBOLS,
        behavior: "Compressed RTF decompression and dictionary/header/flag processing.",
        work_units: &["RP-M2-04"],
    },
    SourceEntry {
        path: "src/lzfu.h",
        line_ref: "",
        symbols: LZFU_H_SYMBOLS,
        behavior: "Compressed RTF declarations.",
        work_units: &["RP-M2-04"],
    },
    SourceEntry {
        path: "src/vbuf.c",
        line_ref: "#L18-L258",
        symbols: VBUF_C_SYMBOLS,
        behavior: "Charset buffer lifecycle, Unicode/code-page conversion, and buffer growth.",
        work_units: &["RP-M1-01", "RP-M1-04", "RP-M2-02", "RP-M6-03"],
    },
    SourceEntry {
        path: "src/vbuf.h",
        line_ref: "",
        symbols: VBUF_H_SYMBOLS,
        behavior: "Charset buffer declarations.",
        work_units: &["RP-M1-01", "RP-M2-02"],
    },
    SourceEntry {
        path: "src/timeconv.c",
        line_ref: "#L1-L34",
        symbols: TIMECONV_C_SYMBOLS,
        behavior: "FILETIME conversion to Unix, textual, and UTC representations with overflow handling.",
        work_units: &["RP-M1-01", "RP-M2-01"],
    },
    SourceEntry {
        path: "src/timeconv.h",
        line_ref: "",
        symbols: TIMECONV_H_SYMBOLS,
        behavior: "FILETIME conversion declarations.",
        work_units: &["RP-M1-01", "RP-M2-01"],
    },
    SourceEntry {
        path: "src/libstrfunc.c",
        line_ref: "#L1-L73",
        symbols: LIBSTRFUNC_C_SYMBOLS,
        behavior: "Base64 encoding and 76-column transfer formatting.",
        work_units: &["RP-M2-02", "RP-M2-04"],
    },
    SourceEntry {
        path: "src/libstrfunc.h",
        line_ref: "",
        symbols: LIBSTRFUNC_H_SYMBOLS,
        behavior: "Base64 helper declarations.",
        work_units: &["RP-M2-02", "RP-M2-04"],
    },
    SourceEntry {
        path: "src/debug.c",
        line_ref: "",
        symbols: DEBUG_C_SYMBOLS,
        behavior: "Debug diagnostics, allocation, and error conventions.",
        work_units: &["RP-M6-03"],
    },
    SourceEntry {
        path: "src/define.h",
        line_ref: "",
        symbols: DEFINE_H_SYMBOLS,
        behavior: "Portability and conditional feature definitions.",
        work_units: &["RP-M0-03", "RP-M1-03", "RP-M6-01"],
    },
    SourceEntry {
        path: "src/common.h",
        line_ref: "",
        symbols: COMMON_H_SYMBOLS,
        behavior: "Common allocation and error declarations.",
        work_units: &["RP-M6-03"],
    },
    SourceEntry {
        path: "src/XGetopt.c",
        line_ref: "",
        symbols: XGETOPT_C_SYMBOLS,
        behavior: "Portable fallback getopt implementation.",
        work_units: &["RP-M3-03"],
    },
    SourceEntry {
        path: "src/XGetopt.h",
        line_ref: "",
        symbols: XGETOPT_H_SYMBOLS,
        behavior: "Portable fallback getopt declarations.",
        work_units: &["RP-M3-03"],
    },
    SourceEntry {
        path: "src/Makefile.am",
        line_ref: "#L20-L95",
        symbols: MAKEFILE_SYMBOLS,
        behavior: "Direct readpst build dependency boundary and linked helper set.",
        work_units: &["RP-M5-04", "RP-M0-03", "RP-M7-03"],
    },
    SourceEntry {
        path: "regression/regression-tests.bash",
        line_ref: "#L1-L167",
        symbols: REGRESSION_SYMBOLS,
        behavior: "Operational profiles, fixture categories, comparison cleanup, and resource/size gates.",
        work_units: &["RP-M0-01", "RP-M0-02", "RP-M3-03", "RP-M5-01", "RP-M7-01", "RP-M7-02"],
    },
    SourceEntry {
        path: "NEWS",
        line_ref: "",
        symbols: NEWS_SYMBOLS,
        behavior: "Historical compatibility fixes and behavior requirements not obvious from current call sites.",
        work_units: &["RP-M4-03", "RP-M6-02", "RP-M7-03"],
    },
    SourceEntry {
        path: "ChangeLog",
        line_ref: "",
        symbols: CHANGELOG_SYMBOLS,
        behavior: "Historical compatibility fixes and release behavior notes.",
        work_units: &["RP-M4-03", "RP-M7-03"],
    },
];

const WORK_UNIT_ANCHORS: &[WorkUnitAnchor] = &[
    WorkUnitAnchor { key: "RP-M0-01", issue: 497, path: "src/readpst.c", symbol: "main" },
    WorkUnitAnchor { key: "RP-M0-02", issue: 498, path: "regression/regression-tests.bash", symbol: "dopst" },
    WorkUnitAnchor { key: "RP-M0-03", issue: 499, path: "src/readpst.c", symbol: "main" },
    WorkUnitAnchor { key: "RP-M0-04", issue: 500, path: "regression/regression-tests.bash", symbol: "profile commands" },
    WorkUnitAnchor { key: "RP-M1-01", issue: 501, path: "src/libpst.c", symbol: "pst_open" },
    WorkUnitAnchor { key: "RP-M1-02", issue: 502, path: "src/readpst.c", symbol: "process" },
    WorkUnitAnchor { key: "RP-M1-03", issue: 503, path: "src/readpst.c", symbol: "process" },
    WorkUnitAnchor { key: "RP-M1-04", issue: 504, path: "src/libpst.c", symbol: "pst_parse_item" },
    WorkUnitAnchor { key: "RP-M2-01", issue: 505, path: "src/readpst.c", symbol: "write_normal_email" },
    WorkUnitAnchor { key: "RP-M2-02", issue: 506, path: "src/readpst.c", symbol: "valid_headers" },
    WorkUnitAnchor { key: "RP-M2-03", issue: 507, path: "src/readpst.c", symbol: "write_inline_attachment" },
    WorkUnitAnchor { key: "RP-M2-04", issue: 508, path: "src/readpst.c", symbol: "write_body_part" },
    WorkUnitAnchor { key: "RP-M3-01", issue: 509, path: "src/readpst.c", symbol: "write_embedded_message" },
    WorkUnitAnchor { key: "RP-M3-02", issue: 510, path: "src/readpst.c", symbol: "write_schedule_part_data" },
    WorkUnitAnchor { key: "RP-M3-03", issue: 511, path: "src/readpst.c", symbol: "main" },
    WorkUnitAnchor { key: "RP-M4-01", issue: 512, path: "src/readpst.c", symbol: "write_vcard" },
    WorkUnitAnchor { key: "RP-M4-02", issue: 513, path: "src/readpst.c", symbol: "write_appointment" },
    WorkUnitAnchor { key: "RP-M4-03", issue: 514, path: "src/readpst.c", symbol: "write_journal" },
    WorkUnitAnchor { key: "RP-M5-01", issue: 515, path: "src/readpst.c", symbol: "create_enter_dir" },
    WorkUnitAnchor { key: "RP-M5-02", issue: 516, path: "src/readpst.c", symbol: "write_separate_attachment" },
    WorkUnitAnchor { key: "RP-M5-03", issue: 517, path: "src/readpst.c", symbol: "mk_thunderbird_dir" },
    WorkUnitAnchor { key: "RP-M5-04", issue: 518, path: "src/msg.cpp", symbol: "write_msg_email" },
    WorkUnitAnchor { key: "RP-M6-01", issue: 519, path: "src/libpst.c", symbol: "pst_open" },
    WorkUnitAnchor { key: "RP-M6-02", issue: 520, path: "src/libpst.c", symbol: "pst_open" },
    WorkUnitAnchor { key: "RP-M6-03", issue: 521, path: "src/vbuf.c", symbol: "buffer growth" },
    WorkUnitAnchor { key: "RP-M7-01", issue: 522, path: "regression/regression-tests.bash", symbol: "dopst" },
    WorkUnitAnchor { key: "RP-M7-02", issue: 523, path: "regression/regression-tests.bash", symbol: "dopst" },
    WorkUnitAnchor { key: "RP-M7-03", issue: 524, path: "NEWS", symbol: "release notes" },
];

const REGRESSION_PROFILES: &[RegressionProfile] = &[
    RegressionProfile { name: "default", command_shape: "readpst fixture.pst", evidence_scope: "ordinary message and canonical extraction" },
    RegressionProfile { name: "separate", command_shape: "readpst -S fixture.pst", evidence_scope: "separate files and attachment paths" },
    RegressionProfile { name: "recursive", command_shape: "readpst -r fixture.pst", evidence_scope: "recursive folder traversal" },
    RegressionProfile { name: "mh", command_shape: "readpst -h fixture.pst", evidence_scope: "MH/rfc822 output" },
    RegressionProfile { name: "kmail", command_shape: "readpst -k fixture.pst", evidence_scope: "KMail output" },
    RegressionProfile { name: "thunderbird", command_shape: "readpst -t fixture.pst", evidence_scope: "Thunderbird sidecars and typed files" },
    RegressionProfile { name: "debug", command_shape: "readpst -d debug.log fixture.pst", evidence_scope: "diagnostic output and bounded logs" },
    RegressionProfile { name: "valgrind-resource", command_shape: "readpst under resource checker", evidence_scope: "resource and leak-budget evidence" },
];

const OUT_OF_SCOPE_UTILITIES: &[&str] = &[
    "lspst",
    "pst2ldif",
    "nick2ldif",
    "pst2dii",
];

pub fn source_manifest() -> SourceManifest {
    SourceManifest {
        repository: UPSTREAM_REPOSITORY,
        revision: READPST_SOURCE_REVISION,
        license_boundary: LICENSE_BOUNDARY_NOTE,
        entries: DIRECT_ENTRIES,
        work_unit_anchors: WORK_UNIT_ANCHORS,
        regression_profiles: REGRESSION_PROFILES,
        out_of_scope_utilities: OUT_OF_SCOPE_UTILITIES,
    }
}

pub fn source_url(path: &str, line_ref: &str) -> String {
    format!(
        "{}{}/{}{}",
        UPSTREAM_BLOB_URL, READPST_SOURCE_REVISION, path, line_ref
    )
}

pub fn check_pinned_revision(observed_revision: &str) -> Result<(), String> {
    if observed_revision == READPST_SOURCE_REVISION {
        Ok(())
    } else {
        Err(format!(
            "upstream_revision_mismatch: expected {} observed {}",
            READPST_SOURCE_REVISION, observed_revision
        ))
    }
}

pub fn stable_drift_report(observed_revision: &str) -> SourceDriftReport {
    let manifest = source_manifest();
    let mut missing_paths = Vec::new();
    let mut duplicate_paths = Vec::new();
    let mut seen = BTreeSet::new();

    for entry in manifest.entries {
        if !seen.insert(entry.path) {
            duplicate_paths.push(entry.path.to_string());
        }
    }
    for required in REQUIRED_DIRECT_PATHS {
        if !seen.contains(required) {
            missing_paths.push((*required).to_string());
        }
    }

    let mut unresolved_work_units = Vec::new();
    for anchor in manifest.work_unit_anchors {
        if manifest
            .entries
            .iter()
            .find(|entry| entry.path == anchor.path)
            .map_or(true, |entry| !entry.symbols.contains(&anchor.symbol))
        {
            unresolved_work_units.push(anchor.key.to_string());
        }
    }

    let status = if observed_revision != READPST_SOURCE_REVISION {
        "revision_mismatch"
    } else if !missing_paths.is_empty()
        || !duplicate_paths.is_empty()
        || !unresolved_work_units.is_empty()
    {
        "manifest_invalid"
    } else {
        "ok"
    };

    let detail = match status {
        "ok" => "pinned source manifest and all work-unit anchors resolve".to_string(),
        "revision_mismatch" => format!(
            "upstream revision changed; review the complete source ledger before updating the oracle (expected {}, observed {})",
            READPST_SOURCE_REVISION, observed_revision
        ),
        _ => "source manifest is incomplete or contains an unresolved work-unit anchor".to_string(),
    };

    SourceDriftReport {
        expected_revision: READPST_SOURCE_REVISION.to_string(),
        observed_revision: observed_revision.to_string(),
        status: status.to_string(),
        missing_paths,
        duplicate_paths,
        unresolved_work_units,
        detail,
    }
}

impl SourceManifest {
    pub fn validate(&self) -> Result<(), String> {
        check_pinned_revision(self.revision)?;
        if self.repository != UPSTREAM_REPOSITORY {
            return Err(format!(
                "upstream_repository_mismatch: expected {} observed {}",
                UPSTREAM_REPOSITORY, self.repository
            ));
        }
        if self.license_boundary != LICENSE_BOUNDARY_NOTE {
            return Err("license_boundary_missing_or_changed".to_string());
        }
        let mut paths = BTreeSet::new();
        for entry in self.entries {
            if entry.path.is_empty() || !paths.insert(entry.path) {
                return Err(format!("duplicate_or_empty_source_path: {}", entry.path));
            }
            if entry.symbols.is_empty() {
                return Err(format!("source_entry_has_no_symbols: {}", entry.path));
            }
            if entry.behavior.is_empty() {
                return Err(format!("source_entry_has_no_behavior: {}", entry.path));
            }
            if entry
                .work_units
                .iter()
                .any(|key| !self.work_unit_anchors.iter().any(|anchor| anchor.key == *key))
            {
                return Err(format!("source_entry_has_unknown_work_unit: {}", entry.path));
            }
            let url = source_url(entry.path, entry.line_ref);
            let expected_prefix = format!("{}{}/{}", UPSTREAM_BLOB_URL, self.revision, entry.path);
            if !url.starts_with(&expected_prefix) {
                return Err(format!("source_url_not_pinned: {}", entry.path));
            }
        }
        for required in REQUIRED_DIRECT_PATHS {
            if !paths.contains(required) {
                return Err(format!("missing_direct_source_path: {}", required));
            }
        }
        let mut work_units = BTreeSet::new();
        for anchor in self.work_unit_anchors {
            if !work_units.insert(anchor.key) {
                return Err(format!("duplicate_work_unit_anchor: {}", anchor.key));
            }
            if self.entries.iter().find(|entry| entry.path == anchor.path).map_or(true, |entry| !entry.symbols.contains(&anchor.symbol)) {
                return Err(format!("unresolved_work_unit_anchor: {}", anchor.key));
            }
            if anchor.issue < 497 || anchor.issue > 524 {
                return Err(format!("work_unit_issue_out_of_range: {}", anchor.key));
            }
        }
        if work_units.len() != 28 {
            return Err(format!("work_unit_anchor_count_mismatch: {}", work_units.len()));
        }
        if self.regression_profiles.len() != 8 {
            return Err(format!(
                "regression_profile_count_mismatch: {}",
                self.regression_profiles.len()
            ));
        }
        if self.out_of_scope_utilities.is_empty() {
            return Err("out_of_scope_utility_boundary_missing".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::readpst_diff::manifest::stable_json;

    #[test]
    fn pinned_manifest_covers_ledger_and_workboard() {
        let manifest = source_manifest();
        manifest.validate().expect("source manifest should validate");
        assert_eq!(manifest.entries.len(), REQUIRED_DIRECT_PATHS.len());
        assert_eq!(manifest.work_unit_anchors.len(), 28);
        assert_eq!(manifest.regression_profiles.len(), 8);
        assert_eq!(stable_drift_report(READPST_SOURCE_REVISION).status, "ok");
        assert!(manifest
            .entries
            .iter()
            .all(|entry| source_url(entry.path, entry.line_ref)
                .starts_with(&format!("{}{}/{}", UPSTREAM_BLOB_URL, READPST_SOURCE_REVISION, entry.path))));
    }

    #[test]
    fn revision_drift_is_actionable_and_stable() {
        let first = check_pinned_revision("newer-upstream-revision").unwrap_err();
        let second = check_pinned_revision("newer-upstream-revision").unwrap_err();
        assert_eq!(first, second);
        assert_eq!(
            first,
            format!(
                "upstream_revision_mismatch: expected {} observed newer-upstream-revision",
                READPST_SOURCE_REVISION
            )
        );
        let report = stable_drift_report("newer-upstream-revision");
        assert_eq!(report.status, "revision_mismatch");
        assert_eq!(report.expected_revision, READPST_SOURCE_REVISION);
        assert_eq!(report.observed_revision, "newer-upstream-revision");
    }

    #[test]
    fn report_serialization_is_deterministic_and_has_no_private_fixture_dependency() {
        let report = stable_drift_report(READPST_SOURCE_REVISION);
        let first = stable_json(&report).expect("report should serialize");
        let second = stable_json(&report).expect("report should serialize");
        assert_eq!(first, second);
        assert!(first.windows(6).all(|window| window != b"GPL code"));
        assert!(first.windows(12).all(|window| window != b"private PST"));
        assert_eq!(report.missing_paths, Vec::<String>::new());
        assert_eq!(report.unresolved_work_units, Vec::<String>::new());
    }
}
