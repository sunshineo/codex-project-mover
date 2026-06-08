use std::fmt::Write as _;

use serde::Serialize;

use crate::app_error::{AppResult, ExitCode, ResultExitCodeExt};
use crate::cli::MoveArgs;
use crate::git_worktree::{
    build_plan_for_existing_project, fsmonitor_report_lines, inspect_fsmonitor_daemons_for_move,
    linked_worktree_move_cwd, GitWorktreeKind,
};
use crate::output::{path_string, print_json, OutputMode, ReferenceOutput};
use crate::pathing::{codex_home_from_arg, normalize_project_path};
use crate::process_guard::{detect_codex_processes, render_process_report};
use crate::scanner::scan_codex_home;

#[derive(Debug, Serialize)]
struct PlanOutput {
    command: &'static str,
    status: &'static str,
    old_path: String,
    new_path: String,
    codex_home: String,
    old_reference_count: usize,
    references: Vec<ReferenceOutput>,
    git_worktree: GitWorktreeOutput,
    fsmonitor: FsmonitorOutput,
    codex_processes: ProcessReportOutput,
}

#[derive(Debug, Serialize)]
struct GitWorktreeOutput {
    kind: &'static str,
    entries_count: usize,
    path_moves: Vec<PathMoveOutput>,
    repair_paths: Vec<String>,
    move_cwd: Option<String>,
}

#[derive(Debug, Serialize)]
struct PathMoveOutput {
    old_path: String,
    new_path: String,
}

#[derive(Debug, Serialize)]
struct FsmonitorOutput {
    checked_paths: Vec<String>,
    running_paths: Vec<String>,
    unsupported_paths: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ProcessReportOutput {
    allow_running_codex: bool,
    count: usize,
    matches: Vec<ProcessOutput>,
}

#[derive(Debug, Serialize)]
struct ProcessOutput {
    pid: u32,
    name: String,
    command: String,
}

pub fn run(args: MoveArgs, output_mode: OutputMode) -> AppResult<()> {
    let old = normalize_project_path(args.old).exit_code(ExitCode::PathValidation)?;
    let new = normalize_project_path(args.new).exit_code(ExitCode::PathValidation)?;
    let codex_home = codex_home_from_arg(args.codex_home).exit_code(ExitCode::PathValidation)?;
    let report = scan_codex_home(&codex_home, &old.to_string_lossy(), &new.to_string_lossy())
        .exit_code(ExitCode::MetadataUpdate)?;
    let old_reference_count = report.old_reference_count();

    let git_plan =
        build_plan_for_existing_project(&old, &new).exit_code(ExitCode::MoveOperation)?;
    let move_cwd = if git_plan.kind == GitWorktreeKind::LinkedWorktree {
        Some(linked_worktree_move_cwd(&git_plan).exit_code(ExitCode::MoveOperation)?)
    } else {
        None
    };
    let fsmonitor_report =
        inspect_fsmonitor_daemons_for_move(&git_plan).exit_code(ExitCode::MoveOperation)?;
    let codex_processes = detect_codex_processes();

    if output_mode.is_json() {
        let output = PlanOutput {
            command: "plan",
            status: "ok",
            old_path: path_string(&old),
            new_path: path_string(&new),
            codex_home: path_string(&codex_home),
            old_reference_count,
            references: report.matches.iter().map(ReferenceOutput::from).collect(),
            git_worktree: GitWorktreeOutput {
                kind: match git_plan.kind {
                    GitWorktreeKind::NotGit => "not_git",
                    GitWorktreeKind::MainWorktree => "main_worktree",
                    GitWorktreeKind::LinkedWorktree => "linked_worktree",
                },
                entries_count: git_plan.entries.len(),
                path_moves: git_plan
                    .path_moves
                    .iter()
                    .map(|path_move| PathMoveOutput {
                        old_path: path_string(&path_move.old_path),
                        new_path: path_string(&path_move.new_path),
                    })
                    .collect(),
                repair_paths: git_plan
                    .repair_paths()
                    .iter()
                    .map(|path| path_string(path))
                    .collect(),
                move_cwd: move_cwd.as_ref().map(|path| path_string(path)),
            },
            fsmonitor: FsmonitorOutput {
                checked_paths: fsmonitor_report
                    .checked_paths
                    .iter()
                    .map(|path| path_string(path))
                    .collect(),
                running_paths: fsmonitor_report
                    .running_paths
                    .iter()
                    .map(|path| path_string(path))
                    .collect(),
                unsupported_paths: fsmonitor_report
                    .unsupported_paths
                    .iter()
                    .map(|path| path_string(path))
                    .collect(),
            },
            codex_processes: ProcessReportOutput {
                allow_running_codex: args.allow_running_codex,
                count: codex_processes.len(),
                matches: codex_processes
                    .iter()
                    .map(|process| ProcessOutput {
                        pid: process.pid,
                        name: process.name.clone(),
                        command: process.command.clone(),
                    })
                    .collect(),
            },
        };
        return print_json(&output);
    }

    println!("Plan: {} -> {}", old.display(), new.display());
    println!("{old_reference_count} old-path reference(s) found");
    for reference in report.matches {
        println!(
            "- {:?}: {} {}",
            reference.surface,
            reference.file.display(),
            reference.location
        );
    }

    match git_plan.kind {
        GitWorktreeKind::NotGit => {
            println!("Git worktree: none detected");
        }
        GitWorktreeKind::MainWorktree => {
            println!("Git worktree: main worktree");
            println!("Git worktree entries: {}", git_plan.entries.len());
            for path_move in &git_plan.path_moves {
                println!(
                    "- Git path move: {} -> {}",
                    path_move.old_path.display(),
                    path_move.new_path.display()
                );
            }
            let repair_paths = git_plan.repair_paths();
            let repair_path_args = repair_paths.iter().fold(String::new(), |mut output, path| {
                write!(&mut output, " {}", path.display()).expect("write to string");
                output
            });
            println!(
                "Git repair: git -C {} worktree repair{}",
                git_plan.new_project_path.display(),
                repair_path_args
            );
        }
        GitWorktreeKind::LinkedWorktree => {
            println!("Git worktree: linked worktree");
            let cwd = move_cwd.as_ref().expect("linked worktree move cwd");
            println!(
                "Git move: git -C {} worktree move {} {}",
                cwd.display(),
                git_plan.project_path.display(),
                git_plan.new_project_path.display()
            );
        }
    }

    for line in fsmonitor_report_lines(&fsmonitor_report) {
        println!("{line}");
    }

    for line in render_process_report(&codex_processes, args.allow_running_codex) {
        println!("{line}");
    }
    Ok(())
}
