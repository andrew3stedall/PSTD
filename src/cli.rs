use clap::{Parser, Subcommand};

use crate::config::ExtractConfig;
use crate::extract::runner::run_extract;

#[derive(Debug, Parser)]
#[command(name = "pstd")]
#[command(about = "PST email data extractor")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Extract PST data into the structured PSTD archive contract.
    Extract {
        /// PST file or directory of PST files.
        #[arg(long)]
        input: std::path::PathBuf,

        /// Output root directory.
        #[arg(long)]
        output: std::path::PathBuf,

        /// Continue when recoverable item failures occur.
        #[arg(long, default_value_t = true)]
        continue_on_error: bool,

        /// Allow writing into an existing output directory.
        #[arg(long, default_value_t = false)]
        overwrite: bool,

        /// Produce inventory and metadata scaffolding without full body/attachment extraction.
        #[arg(long, default_value_t = false)]
        manifest_only: bool,

        /// Archive format. M1 supports tar only.
        #[arg(long, default_value = "tar")]
        archive_format: String,

        /// Structured data format. M1 supports jsonl only.
        #[arg(long, default_value = "jsonl")]
        data_format: String,

        /// Target TAR shard size in MiB.
        #[arg(long, default_value_t = 1024)]
        tar_shard_size_mb: u64,

        /// Progress mode: auto, plain, jsonl, none.
        #[arg(long, default_value = "auto")]
        progress: String,

        /// Log level: error, warn, info, debug, trace.
        #[arg(long, default_value = "info")]
        log_level: String,

        /// Profile: fast, balanced, audit, debug.
        #[arg(long, default_value = "balanced")]
        profile: String,
    },

    /// Inspect command placeholder for future PST structure inspection.
    Inspect {
        #[arg(long)]
        input: std::path::PathBuf,
    },

    /// Print version information.
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
        } => {
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
        Commands::Inspect { input } => {
            println!("PSTD inspect placeholder. PST parser is planned for a later milestone.");
            println!("Input: {}", input.display());
            0
        }
        Commands::Version => {
            println!("pstd {}", env!("CARGO_PKG_VERSION"));
            0
        }
    }
}
