use assert_cmd::Command;
use predicates::str::contains;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command as ProcessCommand,
};
use tempfile::tempdir;

fn mover() -> Command {
    let mut cmd = Command::cargo_bin("codex-project-mover").unwrap();
    cmd.env("CODEX_PROJECT_MOVER_TEST_SKIP_PROCESS_GUARD", "1")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null");
    cmd
}

fn git_available() -> bool {
    ProcessCommand::new("git")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn git_output(cwd: &Path, args: &[&str]) -> std::process::Output {
    ProcessCommand::new("git")
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
        .unwrap()
}

fn git(cwd: &Path, args: &[&str]) {
    let output = git_output(cwd, args);
    assert!(
        output.status.success(),
        "git {:?} failed\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_stdout(cwd: &Path, args: &[&str]) -> String {
    let output = git_output(cwd, args);
    assert!(
        output.status.success(),
        "git {:?} failed\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
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

fn backup_manifest_path(stdout: &[u8]) -> PathBuf {
    let stdout = std::str::from_utf8(stdout).unwrap();
    let backup_dir = stdout
        .lines()
        .find_map(|line| line.strip_prefix("metadata backup: "))
        .unwrap();
    Path::new(backup_dir).join("manifest.json")
}

#[test]
fn apply_repairs_main_worktree_with_nested_linked_worktree() {
    if !git_available() {
        eprintln!(
            "skipping apply_repairs_main_worktree_with_nested_linked_worktree because git is unavailable"
        );
        return;
    }

    let temp = tempdir().unwrap();
    let root = fs::canonicalize(temp.path()).unwrap();
    let home = root.join(".codex");
    let old = root.join("old-project");
    let linked = old.join(".worktrees/feature");
    let new = root.join("new-project");
    let new_linked = new.join(".worktrees/feature");
    let test_trash = root.join("trash");
    fs::create_dir_all(home.join("sessions")).unwrap();
    init_repo(&old);
    fs::create_dir_all(linked.parent().unwrap()).unwrap();
    git(
        &old,
        &["worktree", "add", linked.to_str().unwrap(), "-b", "feature"],
    );
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
        .stdout(contains("Git worktree repair complete"))
        .stdout(contains("move complete: old project folder moved to Trash"));

    assert!(!old.exists());
    assert!(test_trash.join("old-project").exists());
    git(&new, &["status", "--short"]);
    git(&new_linked, &["status", "--short"]);

    let stdout = git_stdout(&new, &["worktree", "list", "--porcelain"]);
    assert!(stdout.contains(new.to_str().unwrap()));
    assert!(stdout.contains(new_linked.to_str().unwrap()));
    assert!(!stdout.contains(old.to_str().unwrap()));
}

#[test]
fn apply_moves_linked_worktree_with_git_worktree_move() {
    if !git_available() {
        eprintln!(
            "skipping apply_moves_linked_worktree_with_git_worktree_move because git is unavailable"
        );
        return;
    }

    let temp = tempdir().unwrap();
    let root = fs::canonicalize(temp.path()).unwrap();
    let home = root.join(".codex");
    let main = root.join("main");
    let old = root.join("linked");
    let new = root.join("moved/linked");
    let test_trash = root.join("trash");
    fs::create_dir_all(home.join("sessions")).unwrap();
    init_repo(&main);
    git(
        &main,
        &["worktree", "add", old.to_str().unwrap(), "-b", "feature"],
    );
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
        .stdout(contains("Git worktree move complete"))
        .stdout(contains(
            "move complete: linked Git worktree moved with git worktree move",
        ));

    assert!(!old.exists());
    assert!(new.exists());
    assert!(!test_trash.join("linked").exists());
    git(&new, &["status", "--short"]);
    assert!(fs::read_to_string(home.join("sessions/thread.jsonl"))
        .unwrap()
        .contains(new.to_str().unwrap()));
}

#[test]
fn rollback_after_linked_worktree_move_preserves_moved_checkout() {
    if !git_available() {
        eprintln!(
            "skipping rollback_after_linked_worktree_move_preserves_moved_checkout because git is unavailable"
        );
        return;
    }

    let temp = tempdir().unwrap();
    let root = fs::canonicalize(temp.path()).unwrap();
    let home = root.join(".codex");
    let main = root.join("main");
    let old = root.join("linked");
    let new = root.join("moved/linked");
    let test_trash = root.join("trash");
    let session = home.join("sessions/thread.jsonl");
    fs::create_dir_all(home.join("sessions")).unwrap();
    init_repo(&main);
    git(
        &main,
        &["worktree", "add", old.to_str().unwrap(), "-b", "feature"],
    );
    fs::write(&session, format!(r#"{{"cwd":"{}"}}"#, old.display())).unwrap();

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
    assert!(
        apply_output.status.success(),
        "apply failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&apply_output.stdout),
        String::from_utf8_lossy(&apply_output.stderr)
    );
    assert!(String::from_utf8_lossy(&apply_output.stdout).contains("Git worktree move complete"));
    let manifest_path = backup_manifest_path(&apply_output.stdout);
    assert!(fs::read_to_string(&session)
        .unwrap()
        .contains(new.to_str().unwrap()));

    mover()
        .env("CODEX_PROJECT_MOVER_TEST_TRASH_DIR", &test_trash)
        .args(["rollback", "--backup", manifest_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("metadata rollback complete"));

    assert!(new.exists());
    assert!(!test_trash.join("linked").exists());
    git(&new, &["status", "--short"]);
    assert!(fs::read_to_string(&session)
        .unwrap()
        .contains(old.to_str().unwrap()));
}
