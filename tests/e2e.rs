use assert_cmd::Command;
use rusqlite::Connection;
use std::fs;
use tempfile::tempdir;

fn mover() -> Command {
    let mut cmd = Command::cargo_bin("codex-project-mover").unwrap();
    cmd.env("CODEX_PROJECT_MOVER_TEST_SKIP_PROCESS_GUARD", "1");
    cmd
}

#[test]
fn relink_only_end_to_end_updates_all_fixture_surfaces() {
    let temp = tempdir().unwrap();
    let home = temp.path().join(".codex");
    let new = temp.path().join("new-project");
    fs::create_dir_all(home.join("sessions")).unwrap();
    fs::create_dir_all(home.join("archived_sessions")).unwrap();
    fs::create_dir_all(home.join("sqlite")).unwrap();
    fs::create_dir_all(&new).unwrap();

    fs::write(
        home.join("sessions/live.jsonl"),
        r#"{"cwd":"/old/project"}"#,
    )
    .unwrap();
    fs::write(
        home.join("archived_sessions/old.jsonl"),
        r#"{"payload":{"cwd":"/old/project"}}"#,
    )
    .unwrap();
    fs::write(
        home.join(".codex-global-state.json"),
        r#"{"roots":["/old/project"]}"#,
    )
    .unwrap();
    fs::write(
        home.join("config.toml"),
        "[desktop]\nopen_target = \"/old/project\"\n",
    )
    .unwrap();

    let state_db = home.join("state_main.sqlite");
    let state_conn = Connection::open(&state_db).unwrap();
    state_conn
        .execute("CREATE TABLE threads (id TEXT PRIMARY KEY, cwd TEXT)", [])
        .unwrap();
    state_conn
        .execute(
            "INSERT INTO threads (id, cwd) VALUES ('t1', '/old/project')",
            [],
        )
        .unwrap();

    let automation_db = home.join("sqlite/codex-dev.db");
    let automation_conn = Connection::open(&automation_db).unwrap();
    automation_conn
        .execute(
            "CREATE TABLE automations (id TEXT PRIMARY KEY, cwds TEXT)",
            [],
        )
        .unwrap();
    automation_conn
        .execute(
            r#"INSERT INTO automations (id, cwds) VALUES ('a1', '["/old/project"]')"#,
            [],
        )
        .unwrap();

    mover()
        .args([
            "apply",
            "--old",
            "/old/project",
            "--new",
            new.to_str().unwrap(),
            "--codex-home",
            home.to_str().unwrap(),
            "--relink-only",
        ])
        .assert()
        .success();

    mover()
        .args([
            "verify",
            "--old",
            "/old/project",
            "--new",
            new.to_str().unwrap(),
            "--codex-home",
            home.to_str().unwrap(),
        ])
        .assert()
        .success();
}
