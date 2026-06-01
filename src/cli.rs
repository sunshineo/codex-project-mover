use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "codex-project-mover")]
#[command(about = "Move a Codex Desktop project and update local Codex metadata")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Plan(MoveArgs),
    Apply(ApplyArgs),
    Verify(MoveArgs),
    Rollback(RollbackArgs),
}

#[derive(Debug, Clone, Args)]
pub struct MoveArgs {
    #[arg(long)]
    pub old: PathBuf,
    #[arg(long)]
    pub new: PathBuf,
    #[arg(long)]
    pub codex_home: Option<PathBuf>,
    #[arg(
        long,
        help = "Proceed even when Codex app-server, CLI, or desktop processes are running"
    )]
    pub allow_running_codex: bool,
}

#[derive(Debug, Clone, Args)]
pub struct ApplyArgs {
    #[arg(long)]
    pub old: PathBuf,
    #[arg(long)]
    pub new: PathBuf,
    #[arg(long)]
    pub codex_home: Option<PathBuf>,
    #[arg(long)]
    pub relink_only: bool,
    #[arg(
        long,
        help = "Proceed even when Codex app-server, CLI, or desktop processes are running"
    )]
    pub allow_running_codex: bool,
}

#[derive(Debug, Clone, Args)]
pub struct RollbackArgs {
    #[arg(long)]
    pub backup: PathBuf,
    #[arg(
        long,
        help = "Proceed even when Codex app-server, CLI, or desktop processes are running"
    )]
    pub allow_running_codex: bool,
}
