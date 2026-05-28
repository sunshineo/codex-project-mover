pub mod backup;
pub mod cli;
pub mod commands;
pub mod discovery;
pub mod error;
pub mod model;
pub mod pathing;
pub mod process_guard;
pub mod project_copy;
pub mod scanner;
pub mod surfaces;
pub mod trash;
pub mod updater;

pub fn run() -> anyhow::Result<()> {
    let cli = <cli::Cli as clap::Parser>::parse();
    match cli.command {
        cli::Command::Plan(args) => commands::plan::run(args),
        cli::Command::Apply(args) => commands::apply::run(args),
        cli::Command::Verify(args) => commands::verify::run(args),
        cli::Command::Rollback(args) => commands::rollback::run(args),
    }
}
