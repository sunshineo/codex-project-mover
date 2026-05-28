use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use sysinfo::System;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub command: String,
}

impl ProcessInfo {
    pub fn new(pid: u32, name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            pid,
            name: name.into(),
            command: command.into(),
        }
    }
}

pub fn assert_no_codex_processes() -> Result<()> {
    if std::env::var("CODEX_PROJECT_MOVER_TEST_SKIP_PROCESS_GUARD").as_deref() == Ok("1") {
        return Ok(());
    }

    let current_pid = std::process::id();
    let mut system = System::new_all();
    system.refresh_all();

    let (processes, app_server_bin) = collect_processes(&system);
    let matches = find_codex_processes(&processes, current_pid);
    if matches.is_empty() {
        return Ok(());
    }

    // Phase 1: try graceful quit of the Codex Desktop app via AppleScript,
    // and stop the managed daemon if one is found.
    println!("Codex-related processes detected. Attempting to stop Codex...");

    let _ = std::process::Command::new("osascript")
        .args(["-e", "tell application \"Codex\" to quit"])
        .status();

    if let Some(ref bin) = app_server_bin {
        let _ = std::process::Command::new(bin)
            .args(["app-server", "daemon", "stop"])
            .output();
    }

    // Poll up to 8 seconds for graceful exit.
    if wait_for_clear(&mut system, current_pid, Duration::from_secs(8)) {
        println!("Codex stopped.");
        return Ok(());
    }

    // Phase 2: SIGTERM any remaining blocked processes.
    system.refresh_all();
    let (refreshed, _) = collect_processes(&system);
    let remaining = find_codex_processes(&refreshed, current_pid);
    if !remaining.is_empty() {
        println!("Sending SIGTERM to remaining Codex processes...");
        signal_processes(&remaining, "-15");
        if wait_for_clear(&mut system, current_pid, Duration::from_secs(5)) {
            println!("Codex stopped.");
            return Ok(());
        }
    }

    // Phase 3: SIGKILL anything still alive.
    system.refresh_all();
    let (refreshed, _) = collect_processes(&system);
    let remaining = find_codex_processes(&refreshed, current_pid);
    if !remaining.is_empty() {
        println!("Sending SIGKILL to remaining Codex processes...");
        signal_processes(&remaining, "-9");
        if wait_for_clear(&mut system, current_pid, Duration::from_secs(5)) {
            println!("Codex stopped.");
            return Ok(());
        }
    }

    // Still blocked — report what's left.
    system.refresh_all();
    let (refreshed, _) = collect_processes(&system);
    let remaining = find_codex_processes(&refreshed, current_pid);
    let rendered = remaining
        .iter()
        .map(|p| format!("pid {}: {} {}", p.pid, p.name, p.command))
        .collect::<Vec<_>>()
        .join("\n");

    bail!(
        "Codex-related processes are still running. Please stop them manually and retry.\n{}",
        rendered
    );
}

fn signal_processes(processes: &[ProcessInfo], signal: &str) {
    for p in processes {
        let _ = std::process::Command::new("kill")
            .args([signal, &p.pid.to_string()])
            .status();
    }
}

fn wait_for_clear(system: &mut System, current_pid: u32, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        std::thread::sleep(Duration::from_millis(500));
        system.refresh_all();
        let (processes, _) = collect_processes(system);
        if find_codex_processes(&processes, current_pid).is_empty() {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
    }
}

/// Collects process info from `system`. Also returns the executable path of the
/// first detected codex app-server process, for use with `app-server daemon stop`.
fn collect_processes(system: &System) -> (Vec<ProcessInfo>, Option<PathBuf>) {
    let mut app_server_bin: Option<PathBuf> = None;

    let processes = system
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

            let lower = command.to_ascii_lowercase();
            if app_server_bin.is_none()
                && lower.contains("codex")
                && lower.contains("app-server")
                && !lower.contains("node_modules")
            {
                if let Some(exe) = process.exe() {
                    app_server_bin = Some(exe.to_path_buf());
                }
            }

            ProcessInfo::new(pid.as_u32(), process.name().to_string_lossy(), command)
        })
        .collect();

    (processes, app_server_bin)
}

fn looks_like_env_var(token: &str) -> bool {
    token
        .split_once('=')
        .is_some_and(|(key, _)| !key.is_empty() && key.chars().all(|c| c.is_ascii_uppercase() || c == '_'))
}

pub fn find_codex_processes(processes: &[ProcessInfo], current_pid: u32) -> Vec<ProcessInfo> {
    processes
        .iter()
        .filter(|process| process.pid != current_pid)
        .filter(|process| {
            let haystack = format!("{} {}", process.name, process.command).to_ascii_lowercase();
            haystack.contains("codex")
                && !haystack.contains("codex-project-mover")
                && !haystack.contains("node_modules")
                && process.name.to_ascii_lowercase() != "extension-host"
        })
        .cloned()
        .collect()
}
