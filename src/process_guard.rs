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

    let processes = system
        .processes()
        .iter()
        .map(|(pid, process)| {
            let command = process
                .cmd()
                .iter()
                .map(|part| part.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ");
            ProcessInfo::new(pid.as_u32(), process.name().to_string_lossy(), command)
        })
        .collect::<Vec<_>>();

    let matches = find_codex_processes(&processes, current_pid);
    if matches.is_empty() {
        return Ok(());
    }

    let rendered = matches
        .iter()
        .map(|process| format!("pid {}: {} {}", process.pid, process.name, process.command))
        .collect::<Vec<_>>()
        .join("\n");

    bail!(
        "Codex-related processes are running. Close Codex and related helper processes, then retry.\n{}",
        rendered
    );
}

pub fn find_codex_processes(processes: &[ProcessInfo], current_pid: u32) -> Vec<ProcessInfo> {
    processes
        .iter()
        .filter(|process| process.pid != current_pid)
        .filter(|process| {
            let haystack = format!("{} {}", process.name, process.command).to_ascii_lowercase();
            haystack.contains("codex") && !haystack.contains("codex-project-mover")
        })
        .cloned()
        .collect()
}
