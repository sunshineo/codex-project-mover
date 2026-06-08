use codex_project_mover::process_guard::{
    assert_no_blocking_codex_processes, find_codex_processes, render_process_report, ProcessInfo,
};

#[test]
fn detects_codex_interfaces_that_can_touch_local_state() {
    let processes = vec![
        ProcessInfo::new(10, "zsh", "/bin/zsh"),
        ProcessInfo::new(11, "Codex", "/Applications/Codex.app/Contents/MacOS/Codex"),
        ProcessInfo::new(
            12,
            "codex",
            "/opt/homebrew/bin/codex exec summarize this repo",
        ),
        ProcessInfo::new(
            13,
            "codex",
            "/Applications/Codex.app/Contents/Resources/codex app-server --listen stdio://",
        ),
    ];

    let matches = find_codex_processes(&processes, 99);

    assert_eq!(matches.len(), 3);
    assert_eq!(matches[0].pid, 11);
    assert_eq!(matches[1].pid, 12);
    assert_eq!(matches[2].pid, 13);
}

#[test]
fn ignores_codex_named_helpers_and_dependency_paths() {
    let processes = vec![
        ProcessInfo::new(
            20,
            "extension-host",
            "/Users/me/.codex/plugins/cache/openai-bundled/chrome/latest/extension-host",
        ),
        ProcessInfo::new(
            21,
            "node",
            "node /repo/node_modules/@openai/codex/dist/openclaw.js",
        ),
        ProcessInfo::new(
            22,
            "browser_crashpad_handler",
            "/Applications/Codex.app/Contents/Frameworks/Codex Framework.framework/Helpers/browser_crashpad_handler",
        ),
        ProcessInfo::new(
            23,
            "Codex (Renderer)",
            "/Applications/Codex.app/Contents/Frameworks/Codex Framework.framework/Helpers/Codex (Renderer).app/Contents/MacOS/Codex (Renderer)",
        ),
        ProcessInfo::new(
            24,
            "Codex (Service)",
            "/Applications/Codex.app/Contents/Frameworks/Codex Framework.framework/Helpers/Codex (Service).app/Contents/MacOS/Codex (Service)",
        ),
    ];

    let matches = find_codex_processes(&processes, 99);

    assert!(matches.is_empty());
}

#[test]
fn ignores_codex_computer_use_helper_with_spaces_in_executable_path() {
    // The Codex Computer Use helper lives in an app bundle whose path contains
    // spaces ("Codex Computer Use.app"). Its real argv[0] basename is
    // SkyComputerUseService, not codex, so it must not be flagged just because
    // an earlier path segment happens to be "Codex". `with_executable` mirrors
    // the precise argv[0] that collect_processes records for the live process.
    let executable =
        "/Users/gordon/.codex/computer-use/Codex Computer Use.app/Contents/MacOS/SkyComputerUseService";
    let processes =
        vec![ProcessInfo::new(30, "SkyComputerUseService", executable).with_executable(executable)];

    let matches = find_codex_processes(&processes, 99);

    assert!(matches.is_empty());
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

#[test]
fn process_guard_reports_matches_without_stopping_processes() {
    let processes = vec![ProcessInfo::new(
        31,
        "codex",
        "/opt/homebrew/bin/codex exec summarize this repo",
    )];

    let error = assert_no_blocking_codex_processes(&processes, 99, false).unwrap_err();

    let rendered = format!("{error:#}");
    assert!(rendered.contains("Codex-related processes are running"));
    assert!(rendered.contains("pid 31: codex"));
    assert!(rendered.contains("--allow-running-codex"));
}

#[test]
fn process_guard_allows_user_bypass() {
    let processes = vec![ProcessInfo::new(
        31,
        "codex",
        "/opt/homebrew/bin/codex exec summarize this repo",
    )];

    assert!(assert_no_blocking_codex_processes(&processes, 99, true).is_ok());
}

#[test]
fn process_report_states_none_detected_when_empty() {
    assert_eq!(
        render_process_report(&[], false),
        vec!["Codex processes: none detected".to_string()]
    );
}

#[test]
fn process_report_warns_apply_would_refuse_by_default() {
    let matches = vec![ProcessInfo::new(
        31,
        "codex",
        "/opt/homebrew/bin/codex exec summarize this repo",
    )];

    let lines = render_process_report(&matches, false);

    assert_eq!(
        lines[0],
        "Codex processes: 1 running — apply would refuse without --allow-running-codex"
    );
    assert_eq!(
        lines[1],
        "- pid 31: codex /opt/homebrew/bin/codex exec summarize this repo"
    );
}

#[test]
fn process_report_notes_apply_would_proceed_when_bypassed() {
    let matches = vec![ProcessInfo::new(
        11,
        "Codex",
        "/Applications/Codex.app/Contents/MacOS/Codex",
    )];

    let lines = render_process_report(&matches, true);

    assert_eq!(
        lines[0],
        "Codex processes: 1 running — --allow-running-codex is set, so apply would proceed"
    );
    assert_eq!(
        lines[1],
        "- pid 11: Codex /Applications/Codex.app/Contents/MacOS/Codex"
    );
}
