//! Headless assembly preview worker — APS-PREVIEW-004 / APS-PREVIEW-002.
//!
//! ```text
//! cargo run --bin bevy_preview_worker -- preview-assembly debug_runs/preview_jobs/<job>.json
//! ```

use std::env;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use proc_A_dine01::preview::{repo_root_from_manifest, run_preview_job};

#[derive(Parser)]
#[command(name = "bevy_preview_worker", about = "Assembly snapshot → PNG (Bevy)")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run job JSON from APS / `rust_engine_mcp.cli preview-assembly`.
    PreviewAssembly {
        /// Path to `preview_jobs/*.json` job file (repo-relative or absolute).
        job_path: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    let repo_root = repo_root_from_manifest();
    if env::set_current_dir(&repo_root).is_err() {
        eprintln!("bevy_preview_worker: could not chdir to {}", repo_root.display());
        std::process::exit(1);
    }

    match cli.command {
        Command::PreviewAssembly { job_path } => {
            let code = run_preview_job(&job_path);
            std::process::exit(code);
        }
    }
}
