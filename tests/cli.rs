use assert_cmd::Command;
use predicates::str::contains;
use std::{fs, path::Path, process::Command as StdCommand};
use tempfile::tempdir;

fn mover() -> Command {
    let mut cmd = Command::cargo_bin("codex-project-mover").unwrap();
    cmd.env("CODEX_PROJECT_MOVER_TEST_SKIP_PROCESS_GUARD", "1");
    cmd
}

fn git_available() -> bool {
    StdCommand::new("git")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn git(cwd: &Path, args: &[&str]) {
    let output = StdCommand::new("git")
        .current_dir(cwd)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .args([
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "init.templateDir=",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {:?} failed\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn init_repo(path: &Path) {
    fs::create_dir_all(path).unwrap();
    git(path, &["init"]);
    git(path, &["config", "user.email", "test@example.com"]);
    git(path, &["config", "user.name", "Test User"]);
    fs::write(path.join("README.md"), "hello\n").unwrap();
    git(path, &["add", "README.md"]);
    git(path, &["commit", "-m", "initial"]);
}

#[test]
fn help_lists_core_subcommands() {
    let mut cmd = mover();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(contains("plan"))
        .stdout(contains("apply"))
        .stdout(contains("verify"))
        .stdout(contains("rollback"));
}

#[test]
fn plan_reports_supported_references() {
    let temp = tempdir().unwrap();
    let home = temp.path().join(".codex");
    fs::create_dir_all(home.join("sessions")).unwrap();
    fs::write(
        home.join("sessions/thread.jsonl"),
        r#"{"cwd":"/old/project"}"#,
    )
    .unwrap();

    mover()
        .args([
            "plan",
            "--old",
            "/old/project",
            "--new",
            "/new/project",
            "--codex-home",
            home.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("1 old-path reference"))
        .stdout(contains("JsonlCwd"));
}

#[test]
fn verify_fails_when_old_references_remain() {
    let temp = tempdir().unwrap();
    let home = temp.path().join(".codex");
    fs::create_dir_all(home.join("sessions")).unwrap();
    fs::write(
        home.join("sessions/thread.jsonl"),
        r#"{"cwd":"/old/project"}"#,
    )
    .unwrap();

    mover()
        .args([
            "verify",
            "--old",
            "/old/project",
            "--new",
            "/new/project",
            "--codex-home",
            home.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(contains(
            "verification failed: 1 old-path reference remains",
        ));
}

#[test]
fn verify_passes_when_old_references_are_gone_and_new_references_exist() {
    let temp = tempdir().unwrap();
    let home = temp.path().join(".codex");
    fs::create_dir_all(home.join("sessions")).unwrap();
    fs::write(
        home.join("sessions/thread.jsonl"),
        r#"{"cwd":"/new/project"}"#,
    )
    .unwrap();

    mover()
        .args([
            "verify",
            "--old",
            "/old/project",
            "--new",
            "/new/project",
            "--codex-home",
            home.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("verification passed"))
        .stdout(contains("1 new-path reference"));
}

#[test]
fn apply_relink_only_updates_metadata_when_old_is_missing_and_new_exists() {
    let temp = tempdir().unwrap();
    let home = temp.path().join(".codex");
    let new = temp.path().join("new-project");
    fs::create_dir_all(home.join("sessions")).unwrap();
    fs::create_dir_all(&new).unwrap();
    fs::write(
        home.join("sessions/thread.jsonl"),
        r#"{"cwd":"/old/project"}"#,
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
        .success()
        .stdout(contains("metadata backup:"))
        .stdout(contains("updated 1 metadata reference"));

    assert!(fs::read_to_string(home.join("sessions/thread.jsonl"))
        .unwrap()
        .contains(new.to_str().unwrap()));
}

#[test]
fn apply_relink_only_fails_when_old_exists() {
    let temp = tempdir().unwrap();
    let home = temp.path().join(".codex");
    let old = temp.path().join("old-project");
    let new = temp.path().join("new-project");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&old).unwrap();
    fs::create_dir_all(&new).unwrap();

    mover()
        .args([
            "apply",
            "--old",
            old.to_str().unwrap(),
            "--new",
            new.to_str().unwrap(),
            "--codex-home",
            home.to_str().unwrap(),
            "--relink-only",
        ])
        .assert()
        .failure()
        .stderr(contains("relink-only requires old path to not exist"));
}

#[test]
fn apply_normal_move_copies_updates_and_moves_old_folder_to_test_trash() {
    let temp = tempdir().unwrap();
    let home = temp.path().join(".codex");
    let old = temp.path().join("old-project");
    let new = temp.path().join("nested/new-project");
    let test_trash = temp.path().join("trash");
    fs::create_dir_all(home.join("sessions")).unwrap();
    fs::create_dir_all(old.join("src")).unwrap();
    fs::write(old.join("src/main.rs"), "fn main() {}\n").unwrap();
    fs::write(
        home.join("sessions/thread.jsonl"),
        format!(r#"{{"cwd":"{}"}}"#, old.display()),
    )
    .unwrap();

    mover()
        .env("CODEX_PROJECT_MOVER_TEST_TRASH_DIR", &test_trash)
        .args([
            "apply",
            "--old",
            old.to_str().unwrap(),
            "--new",
            new.to_str().unwrap(),
            "--codex-home",
            home.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("metadata backup:"))
        .stdout(contains("move complete: old project folder moved to Trash"));

    assert!(new.join("src/main.rs").exists());
    assert!(!old.exists());
    assert!(test_trash.join("old-project").exists());
    assert!(fs::read_to_string(home.join("sessions/thread.jsonl"))
        .unwrap()
        .contains(new.to_str().unwrap()));
}

#[test]
fn rollback_restores_metadata_from_manifest() {
    let temp = tempdir().unwrap();
    let home = temp.path().join(".codex");
    let new = temp.path().join("new-project");
    fs::create_dir_all(home.join("sessions")).unwrap();
    fs::create_dir_all(&new).unwrap();
    let jsonl = home.join("sessions/thread.jsonl");
    fs::write(&jsonl, r#"{"cwd":"/old/project"}"#).unwrap();

    let apply_output = mover()
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
        .output()
        .unwrap();
    assert!(apply_output.status.success());

    let stdout = String::from_utf8(apply_output.stdout).unwrap();
    let backup_dir = stdout
        .lines()
        .find_map(|line| line.strip_prefix("metadata backup: "))
        .unwrap();

    fs::write(&jsonl, r#"{"cwd":"broken"}"#).unwrap();

    mover()
        .args([
            "rollback",
            "--backup",
            &format!("{}/manifest.json", backup_dir),
        ])
        .assert()
        .success()
        .stdout(contains("metadata rollback complete"));

    assert_eq!(
        fs::read_to_string(&jsonl).unwrap(),
        r#"{"cwd":"/old/project"}"#
    );
}

#[test]
fn rollback_after_normal_move_removes_created_new_folder() {
    let temp = tempdir().unwrap();
    let home = temp.path().join(".codex");
    let old = temp.path().join("old-project");
    let new = temp.path().join("nested/new-project");
    let test_trash = temp.path().join("trash");
    fs::create_dir_all(home.join("sessions")).unwrap();
    fs::create_dir_all(old.join("src")).unwrap();
    fs::write(old.join("src/main.rs"), "fn main() {}\n").unwrap();
    fs::write(
        home.join("sessions/thread.jsonl"),
        format!(r#"{{"cwd":"{}"}}"#, old.display()),
    )
    .unwrap();

    let apply_output = mover()
        .env("CODEX_PROJECT_MOVER_TEST_TRASH_DIR", &test_trash)
        .args([
            "apply",
            "--old",
            old.to_str().unwrap(),
            "--new",
            new.to_str().unwrap(),
            "--codex-home",
            home.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(apply_output.status.success());

    let stdout = String::from_utf8(apply_output.stdout).unwrap();
    let backup_dir = stdout
        .lines()
        .find_map(|line| line.strip_prefix("metadata backup: "))
        .unwrap();

    mover()
        .env("CODEX_PROJECT_MOVER_TEST_TRASH_DIR", &test_trash)
        .args([
            "rollback",
            "--backup",
            &format!("{}/manifest.json", backup_dir),
        ])
        .assert()
        .success()
        .stdout(contains("removed created new project folder"))
        .stdout(contains("metadata rollback complete"));

    assert!(!new.exists());
    assert!(test_trash.join("new-project").exists());
}

#[test]
fn plan_reports_no_git_worktree_repair_for_plain_folder() {
    let temp = tempdir().unwrap();
    let home = temp.path().join(".codex");
    let old = temp.path().join("old-project");
    let new = temp.path().join("new-project");
    fs::create_dir_all(home.join("sessions")).unwrap();
    fs::create_dir_all(&old).unwrap();

    mover()
        .args([
            "plan",
            "--old",
            old.to_str().unwrap(),
            "--new",
            new.to_str().unwrap(),
            "--codex-home",
            home.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("Git worktree: none detected"));
}

#[test]
fn plan_reports_main_worktree_repair_paths() {
    if !git_available() {
        eprintln!("skipping plan_reports_main_worktree_repair_paths because git is unavailable");
        return;
    }

    let temp = tempdir().unwrap();
    let root = fs::canonicalize(temp.path()).unwrap();
    let home = root.join(".codex");
    let old = root.join("repo");
    let linked_parent = old.join(".worktrees");
    let linked = linked_parent.join("feature");
    let new = root.join("moved/repo");
    let new_linked = new.join(".worktrees/feature");
    fs::create_dir_all(&home).unwrap();
    init_repo(&old);
    fs::create_dir_all(&linked_parent).unwrap();
    git(
        &old,
        &["worktree", "add", linked.to_str().unwrap(), "-b", "feature"],
    );

    mover()
        .args([
            "plan",
            "--old",
            old.to_str().unwrap(),
            "--new",
            new.to_str().unwrap(),
            "--codex-home",
            home.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains(format!(
            "Git repair: git -C {} worktree repair {}",
            new.display(),
            new_linked.display()
        )));
}

#[test]
fn plan_reports_linked_worktree_move_cwd() {
    if !git_available() {
        eprintln!("skipping plan_reports_linked_worktree_move_cwd because git is unavailable");
        return;
    }

    let temp = tempdir().unwrap();
    let root = fs::canonicalize(temp.path()).unwrap();
    let home = root.join(".codex");
    let main = root.join("repo");
    let linked = root.join("linked");
    let new = root.join("moved-linked");
    fs::create_dir_all(&home).unwrap();
    init_repo(&main);
    git(
        &main,
        &["worktree", "add", linked.to_str().unwrap(), "-b", "feature"],
    );

    mover()
        .args([
            "plan",
            "--old",
            linked.to_str().unwrap(),
            "--new",
            new.to_str().unwrap(),
            "--codex-home",
            home.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains(format!(
            "Git move: git -C {} worktree move {} {}",
            main.display(),
            linked.display(),
            new.display()
        )));
}
