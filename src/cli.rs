use clap::{Args, Parser, Subcommand};

use crate::config::{ExtractConfig, ReadpstPolicy};
use crate::engine::batch::{run_batch, BatchConfig};
use crate::engine::runner::run_extract;
use crate::pst::inspect::inspect_pst;

#[derive(Debug, Parser)]
#[command(name = "pstd")]
#[command(about = "PST email data extractor")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Clone, Args)]
pub struct ReadpstArgs {
    /// Named output family; non-canonical families fail explicitly until their adapter lands.
    #[arg(long, default_value = "canonical")]
    output_profile: String,
    #[arg(long)]
    fallback_charset: Option<String>,
    #[arg(long, default_value_t = false)]
    prefer_utf8: bool,
    #[arg(long, default_value_t = false)]
    include_deleted: bool,
    #[arg(long, default_value_t = false)]
    include_associated: bool,
    #[arg(long, default_value = "all")]
    item_types: String,
    #[arg(long)]
    attachment_extensions: Option<String>,
    #[arg(long, default_value_t = false)]
    no_synthetic_rtf: bool,
    #[arg(long, default_value_t = 1)]
    jobs: u16,
    #[arg(long, default_value = "info")]
    diagnostics: String,
    #[arg(long, default_value = "suffix")]
    collision: String,
}

impl ReadpstArgs {
    fn policy(&self, overwrite: bool) -> Result<ReadpstPolicy, String> {
        ReadpstPolicy::from_flags(
            &self.output_profile,
            self.fallback_charset.clone(),
            self.prefer_utf8,
            self.include_deleted,
            self.include_associated,
            &self.item_types,
            self.attachment_extensions.as_deref(),
            !self.no_synthetic_rtf,
            self.jobs,
            &self.diagnostics,
            &self.collision,
            overwrite,
        )
        .map_err(|error| error.to_string())
    }
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Extract {
        #[arg(long)]
        input: std::path::PathBuf,
        #[arg(long)]
        output: std::path::PathBuf,
        #[arg(long, default_value_t = true)]
        continue_on_error: bool,
        #[arg(long, default_value_t = false)]
        overwrite: bool,
        #[arg(long, default_value_t = false)]
        manifest_only: bool,
        #[arg(long, default_value = "tar")]
        archive_format: String,
        #[arg(long, default_value = "jsonl")]
        data_format: String,
        #[arg(long, default_value_t = 1024)]
        tar_shard_size_mb: u64,
        #[arg(long, default_value = "auto")]
        progress: String,
        #[arg(long, default_value = "info")]
        log_level: String,
        #[arg(long, default_value = "balanced")]
        profile: String,
        #[command(flatten)]
        readpst: ReadpstArgs,
    },
    Batch {
        #[arg(long)]
        input: std::path::PathBuf,
        #[arg(long)]
        output: std::path::PathBuf,
        #[arg(long, default_value_t = true)]
        recursive: bool,
        #[arg(long, default_value_t = true)]
        continue_on_error: bool,
        #[arg(long, default_value_t = false)]
        overwrite: bool,
        #[arg(long, default_value_t = false)]
        manifest_only: bool,
        #[arg(long, default_value = "tar")]
        archive_format: String,
        #[arg(long, default_value = "jsonl")]
        data_format: String,
        #[arg(long, default_value_t = 1024)]
        tar_shard_size_mb: u64,
        #[arg(long, default_value = "auto")]
        progress: String,
        #[arg(long, default_value = "info")]
        log_level: String,
        #[arg(long, default_value = "balanced")]
        profile: String,
        #[command(flatten)]
        readpst: ReadpstArgs,
    },
    Inspect {
        #[arg(long)]
        input: std::path::PathBuf,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    Version,
}

pub fn run() -> i32 {
    let cli = Cli::parse();
    match cli.command {
        Commands::Extract {
            input,
            output,
            continue_on_error,
            overwrite,
            manifest_only,
            archive_format,
            data_format,
            tar_shard_size_mb,
            progress,
            log_level,
            profile,
            readpst,
        } => {
            let readpst = match readpst.policy(overwrite) {
                Ok(policy) => policy,
                Err(error) => {
                    eprintln!("PSTD extract failed: {error}");
                    return 1;
                }
            };
            let config = ExtractConfig {
                input,
                output,
                continue_on_error,
                overwrite,
                manifest_only,
                archive_format,
                data_format,
                tar_shard_size_mb,
                progress,
                log_level,
                profile,
                readpst,
            };
            match run_extract(config) {
                Ok(summary) => {
                    println!("PSTD extract completed: {}", summary.status);
                    0
                }
                Err(err) => {
                    eprintln!("PSTD extract failed: {err}");
                    1
                }
            }
        }
        Commands::Batch {
            input,
            output,
            recursive,
            continue_on_error,
            overwrite,
            manifest_only,
            archive_format,
            data_format,
            tar_shard_size_mb,
            progress,
            log_level,
            profile,
            readpst,
        } => {
            let readpst = match readpst.policy(overwrite) {
                Ok(policy) => policy,
                Err(error) => {
                    eprintln!("PSTD batch failed: {error}");
                    return 1;
                }
            };
            let config = BatchConfig {
                input,
                output,
                recursive,
                continue_on_error,
                overwrite,
                manifest_only,
                archive_format,
                data_format,
                tar_shard_size_mb,
                progress,
                log_level,
                profile,
                readpst,
            };
            match run_batch(config) {
                Ok(summary) => {
                    println!(
                        "PSTD batch {status}: discovered={discovered}, attempted={attempted}, completed={completed}, partial={partial}, failed={failed}, skipped={skipped}, not_run={not_run}",
                        status = summary.status,
                        discovered = summary.pst_discovered,
                        attempted = summary.pst_attempted,
                        completed = summary.pst_completed,
                        partial = summary.pst_partial,
                        failed = summary.pst_failed,
                        skipped = summary.pst_skipped,
                        not_run = summary.pst_not_run,
                    );
                    if summary.pst_failed == 0 {
                        0
                    } else {
                        1
                    }
                }
                Err(err) => {
                    eprintln!("PSTD batch failed: {err}");
                    1
                }
            }
        }
        Commands::Inspect { input, json } => match inspect_pst(&input) {
            Ok(summary) => {
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&summary).unwrap_or_else(|_| "{}".to_string())
                    );
                } else {
                    println!("{}", summary.to_human_text());
                }
                0
            }
            Err(err) => {
                eprintln!("PSTD inspect failed: {err}");
                1
            }
        },
        Commands::Version => {
            println!("pstd {}", env!("CARGO_PKG_VERSION"));
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use crate::config::{CollisionPolicy, DiagnosticsPolicy, OutputProfile};
    use clap::Parser;

    #[test]
    fn translates_readpst_policy_flags_deterministically() {
        let cli = Cli::try_parse_from([
            "pstd",
            "extract",
            "--input",
            "fixture.pst",
            "--output",
            "out",
            "--include-deleted",
            "--include-associated",
            "--item-types",
            "c",
            "--attachment-extensions",
            ".DOC,txt,.doc",
            "--jobs",
            "4",
            "--diagnostics",
            "debug",
            "--collision",
            "fail",
            "--no-synthetic-rtf",
        ])
        .expect("CLI should parse");
        let super::Commands::Extract {
            readpst, overwrite, ..
        } = cli.command
        else {
            panic!("expected extract command");
        };
        let policy = readpst.policy(overwrite).expect("policy should validate");
        assert!(policy.include_deleted);
        assert!(policy.include_associated);
        assert_eq!(
            policy.item_type_filter,
            crate::pst::item_routing::ItemTypeFilter::Contact
        );
        assert_eq!(policy.attachment_extensions, ["doc", "txt"]);
        assert_eq!(policy.jobs, 4);
        assert_eq!(policy.diagnostics, DiagnosticsPolicy::Debug);
        assert_eq!(policy.collision, CollisionPolicy::Fail);
        assert!(!policy.emit_synthetic_rtf);
        assert_eq!(policy.output_profile, OutputProfile::Canonical);
    }

    #[test]
    fn accepts_msg_output_profile() {
        let cli = Cli::try_parse_from([
            "pstd",
            "extract",
            "--input",
            "fixture.pst",
            "--output",
            "out",
            "--output-profile",
            "msg",
        ])
        .expect("CLI should parse");
        let super::Commands::Extract {
            readpst, overwrite, ..
        } = cli.command
        else {
            panic!("expected extract command");
        };
        let policy = readpst.policy(overwrite).expect("msg should be supported");
        assert_eq!(policy.output_profile, OutputProfile::Msg);
    }
}
