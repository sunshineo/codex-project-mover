use std::fs;

use codex_project_mover::surfaces::jsonl::{scan_jsonl_file, update_jsonl_file};
use tempfile::tempdir;

#[test]
fn scans_only_structured_cwd_fields_equal_to_old_path() {
    let temp = tempdir().unwrap();
    let file = temp.path().join("thread.jsonl");
    fs::write(
        &file,
        r#"{"cwd":"/old/project","message":"do not edit /old/project in text"}"#.to_owned()
            + "\n"
            + r#"{"payload":{"cwd":"/old/project/subdir"}}"#
            + "\n"
            + r#"{"payload":{"cwd":"/old/project"}}"#
            + "\n",
    )
    .unwrap();

    let matches = scan_jsonl_file(&file, "/old/project", "/new/project").unwrap();

    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].location, "line 1 /cwd");
    assert_eq!(matches[1].location, "line 3 /payload/cwd");
}

#[test]
fn updates_only_matching_cwd_fields() {
    let temp = tempdir().unwrap();
    let file = temp.path().join("thread.jsonl");
    fs::write(
        &file,
        r#"{"cwd":"/old/project","message":"do not edit /old/project in text"}"#.to_owned()
            + "\n"
            + r#"{"payload":{"cwd":"/old/project/subdir"}}"#
            + "\n",
    )
    .unwrap();

    let count = update_jsonl_file(&file, "/old/project", "/new/project").unwrap();
    let updated = fs::read_to_string(&file).unwrap();

    assert_eq!(count, 1);
    assert!(updated.contains(r#""cwd":"/new/project""#));
    assert!(updated.contains(r#""message":"do not edit /old/project in text""#));
    assert!(updated.contains(r#""cwd":"/old/project/subdir""#));
}
