use codex_project_mover::process_guard::{find_codex_processes, ProcessInfo};

#[test]
fn detects_codex_in_process_name_or_command() {
    let processes = vec![
        ProcessInfo::new(10, "zsh", "/bin/zsh"),
        ProcessInfo::new(11, "Codex", "/Applications/Codex.app/Contents/MacOS/Codex"),
        ProcessInfo::new(12, "node", "node /tmp/codex app-server"),
    ];

    let matches = find_codex_processes(&processes, 99);

    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].pid, 11);
    assert_eq!(matches[1].pid, 12);
}

#[test]
fn excludes_current_mover_process() {
    let processes = vec![ProcessInfo::new(
        22,
        "codex-project-mover",
        "/usr/local/bin/codex-project-mover apply",
    )];

    let matches = find_codex_processes(&processes, 22);

    assert!(matches.is_empty());
}
