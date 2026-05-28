use std::path::PathBuf;

use codex_project_mover::pathing::{codex_home_from_arg, normalize_project_path};

#[test]
fn normalize_project_path_makes_relative_path_absolute() {
    let normalized = normalize_project_path(PathBuf::from("relative/project")).unwrap();

    assert!(normalized.is_absolute());
    assert!(normalized.ends_with("relative/project"));
}

#[test]
fn normalize_project_path_removes_dot_components() {
    let normalized = normalize_project_path(PathBuf::from("/tmp/./old/../old/project/")).unwrap();

    assert_eq!(normalized, PathBuf::from("/tmp/old/project"));
}

#[test]
fn codex_home_uses_explicit_arg_first() {
    let home = codex_home_from_arg(Some(PathBuf::from("/tmp/custom-codex"))).unwrap();

    assert_eq!(home, PathBuf::from("/tmp/custom-codex"));
}
