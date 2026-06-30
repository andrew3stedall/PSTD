use clap::{Parser, Subcommand};

use crate::config::ExtractConfig;
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
