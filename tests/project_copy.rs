use std::fs;

use codex_project_mover::project_copy::{copy_project_tree, verify_project_tree};
use tempfile::tempdir;

#[test]
fn copies_and_verifies_project_tree() {
    let temp = tempdir().unwrap();
    let old = temp.path().join("old-project");
    let new = temp.path().join("nested/new-project");
    fs::create_dir_all(old.join("src")).unwrap();
    fs::write(old.join("src/main.rs"), "fn main() {}\n").unwrap();

    copy_project_tree(&old, &new).unwrap();
    verify_project_tree(&old, &new).unwrap();

    assert_eq!(
        fs::read_to_string(new.join("src/main.rs")).unwrap(),
        "fn main() {}\n"
    );
}

#[cfg(unix)]
#[test]
fn verifies_project_tree_ignores_git_fsmonitor_socket() {
    let temp = tempdir().unwrap();
    let old = temp.path().join("old-project");
    let new = temp.path().join("new-project");
    let socket = old.join(".git/fsmonitor--daemon.ipc");
    fs::create_dir_all(socket.parent().unwrap()).unwrap();
    fs::write(old.join("README.md"), "hello\n").unwrap();
    let _listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();

    copy_project_tree(&old, &new).unwrap();

    assert!(!new.join(".git/fsmonitor--daemon.ipc").exists());
    verify_project_tree(&old, &new).unwrap();
}
