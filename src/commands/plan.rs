use anyhow::Result;

use crate::cli::MoveArgs;
use crate::pathing::{codex_home_from_arg, normalize_project_path};
use crate::process_guard::assert_no_codex_processes;
use crate::scanner::scan_codex_home;

pub fn run(args: MoveArgs) -> Result<()> {
    assert_no_codex_processes()?;
    let old = normalize_project_path(args.old)?;
    let new = normalize_project_path(args.new)?;
    let codex_home = codex_home_from_arg(args.codex_home)?;
    let report = scan_codex_home(&codex_home, &old.to_string_lossy(), &new.to_string_lossy())?;

    println!("Plan: {} -> {}", old.display(), new.display());
    println!(
        "{} old-path reference(s) found",
        report.old_reference_count()
    );
    for reference in report.matches {
        println!(
            "- {:?}: {} {}",
            reference.surface,
            reference.file.display(),
            reference.location
        );
    }
    Ok(())
}
