use std::fs;

use codex_project_mover::surfaces::global_state::{scan_global_state, update_global_state};
use tempfile::tempdir;

#[test]
fn scans_and_updates_only_exact_string_values_and_exact_path_keys() {
    let temp = tempdir().unwrap();
    let file = temp.path().join(".codex-global-state.json");
    fs::write(
        &file,
        r#"{
          "workspaceRoots":["/old/project","/old/project/subdir"],
          "electron-workspace-root-labels":{"/old/project":"Old Project","/old/project/subdir":"Nested"},
          "nested":{"cwd":"/old/project"},
          "message":"do not edit /old/project inside text"
        }"#,
    )
    .unwrap();

    let matches = scan_global_state(&file, "/old/project", "/new/project").unwrap();
    let mut locations = matches
        .iter()
        .map(|m| m.location.as_str())
        .collect::<Vec<_>>();
    locations.sort();
    assert_eq!(matches.len(), 3);
    assert_eq!(
        locations,
        vec![
            "/electron-workspace-root-labels/~1old~1project",
            "/nested/cwd",
            "/workspaceRoots/0",
        ]
    );

    let changed = update_global_state(&file, "/old/project", "/new/project").unwrap();
    let updated = fs::read_to_string(&file).unwrap();

    assert_eq!(changed, 3);
    assert!(updated.contains(r#""/new/project""#));
    assert!(updated.contains(r#""/new/project": "Old Project""#));
    assert!(updated.contains(r#""/old/project/subdir""#));
    assert!(updated.contains(r#""do not edit /old/project inside text""#));
}
