use anyhow::{bail, Result};
use sysinfo::System;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    /// The executable (argv[0]) kept whole, so a path containing spaces (e.g. a
    /// macOS app bundle) is never mistaken for several tokens during detection.
    pub executable: String,
    pub command: String,
}

impl ProcessInfo {
    pub fn new(pid: u32, name: impl Into<String>, command: impl Into<String>) -> Self {
        let command = command.into();
        // Best-effort default for synthetic/test data: the first
        // whitespace-delimited token. Real processes overwrite this with the
        // precise argv[0] via `with_executable`.
        let executable = command
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_string();
        Self {
            pid,
            name: name.into(),
            executable,
            command,
        }
    }

    pub fn with_executable(mut self, executable: impl Into<String>) -> Self {
        self.executable = executable.into();
        self
    }
}

const TEST_SKIP_GUARD_ENV: &str = "CODEX_PROJECT_MOVER_TEST_SKIP_PROCESS_GUARD";

fn guard_skipped_for_tests() -> bool {
    std::env::var(TEST_SKIP_GUARD_ENV).as_deref() == Ok("1")
}

pub fn assert_no_codex_processes(allow_running_codex: bool) -> Result<()> {
    if guard_skipped_for_tests() || allow_running_codex {
        return Ok(());
    }
    bail_on_codex_processes(&detect_codex_processes())
}

/// Scan the live system for Codex processes that could race with a move,
/// returning the matches instead of failing. `plan` uses this to surface the
/// condition in its dry-run report without blocking. Honors the test skip env
/// var so dry-run output stays deterministic in tests.
pub fn detect_codex_processes() -> Vec<ProcessInfo> {
    if guard_skipped_for_tests() {
        return Vec::new();
    }

    let current_pid = std::process::id();
    let mut system = System::new_all();
    system.refresh_all();

    let processes = collect_processes(&system);
    find_codex_processes(&processes, current_pid)
}

/// Build the lines `plan` prints for its dry-run Codex-process report. An empty
/// slice yields the "none detected" status; otherwise a header stating what
/// `apply` would do (refuse, or proceed under `--allow-running-codex`) followed
/// by one line per detected process.
pub fn render_process_report(matches: &[ProcessInfo], allow_running_codex: bool) -> Vec<String> {
    if matches.is_empty() {
        return vec!["Codex processes: none detected".to_string()];
    }

    let consequence = if allow_running_codex {
        "--allow-running-codex is set, so apply would proceed"
    } else {
        "apply would refuse without --allow-running-codex"
    };
    let mut lines = vec![format!(
        "Codex processes: {} running — {}",
        matches.len(),
        consequence
    )];
    lines.extend(matches.iter().map(|process| {
        format!(
            "- pid {}: {} {}",
            process.pid, process.name, process.command
        )
    }));
    lines
}

fn collect_processes(system: &System) -> Vec<ProcessInfo> {
    system
        .processes()
        .iter()
        .map(|(pid, process)| {
            let command = process
                .cmd()
                .iter()
                .map(|part| part.to_string_lossy())
                .filter(|part| !looks_like_env_var(part))
                .collect::<Vec<_>>()
                .join(" ");

            // argv[0] is the executable as a single unit; keep it intact so a
            // path with spaces survives instead of being split on them.
            let executable = process
                .cmd()
                .first()
                .map(|part| part.to_string_lossy().into_owned())
                .unwrap_or_default();

            ProcessInfo::new(pid.as_u32(), process.name().to_string_lossy(), command)
                .with_executable(executable)
        })
        .collect()
}

fn looks_like_env_var(token: &str) -> bool {
    token.split_once('=').is_some_and(|(key, _)| {
        !key.is_empty() && key.chars().all(|c| c.is_ascii_uppercase() || c == '_')
    })
}

pub fn find_codex_processes(processes: &[ProcessInfo], current_pid: u32) -> Vec<ProcessInfo> {
    processes
        .iter()
        .filter(|process| process.pid != current_pid)
        .filter(|process| is_blocking_codex_process(process))
        .cloned()
        .collect()
}

pub fn assert_no_blocking_codex_processes(
    processes: &[ProcessInfo],
    current_pid: u32,
    allow_running_codex: bool,
) -> Result<()> {
    if allow_running_codex {
        return Ok(());
    }
    bail_on_codex_processes(&find_codex_processes(processes, current_pid))
}

fn bail_on_codex_processes(matches: &[ProcessInfo]) -> Result<()> {
    if matches.is_empty() {
        return Ok(());
    }

    let rendered = matches
        .iter()
        .map(|process| format!("pid {}: {} {}", process.pid, process.name, process.command))
        .collect::<Vec<_>>()
        .join("\n");

    bail!(
        "Codex-related processes are running. This tool edits local Codex state under CODEX_HOME, so concurrent Codex app-server, CLI, or desktop processes may overwrite or race with the move.\nClose those processes and retry, or rerun with --allow-running-codex if you know they are unrelated to this project.\n{}",
        rendered
    );
}

fn is_blocking_codex_process(process: &ProcessInfo) -> bool {
    let command = process.command.to_ascii_lowercase();
    if command.contains("codex-project-mover") || command.contains("node_modules") {
        return false;
    }

    // Block only the Codex surfaces that can plausibly read or write the same
    // CODEX_HOME state this tool mutates. `codex exec` can persist rollout files
    // and initialize state DBs in-process, so a separate `codex app-server`
    // process is not the only risky shape. The Electron helpers, crashpad
    // handlers, browser extension host, and dependency paths are intentionally
    // ignored to avoid false positives from tools such as OpenClaw.
    is_main_desktop_process(process)
        || is_app_server_process(process)
        || is_standalone_codex_cli_process(process)
}

fn is_main_desktop_process(process: &ProcessInfo) -> bool {
    process.name.eq_ignore_ascii_case("Codex")
        && process.command.contains("/Codex.app/Contents/MacOS/Codex")
}

fn is_app_server_process(process: &ProcessInfo) -> bool {
    executable_basename(&process.executable).eq_ignore_ascii_case("codex")
        && command_words(&process.command)
            .iter()
            .any(|word| word.eq_ignore_ascii_case("app-server"))
}

fn is_standalone_codex_cli_process(process: &ProcessInfo) -> bool {
    let command = process.command.to_ascii_lowercase();
    if command.contains("/codex.app/contents/frameworks/") {
        return false;
    }

    process.name.eq_ignore_ascii_case("codex")
        || executable_basename(&process.executable).eq_ignore_ascii_case("codex")
}

fn executable_basename(executable: &str) -> &str {
    executable.rsplit(['/', '\\']).next().unwrap_or(executable)
}

fn command_words(command: &str) -> Vec<&str> {
    command.split_whitespace().collect()
}
