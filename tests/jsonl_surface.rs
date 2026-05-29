use std::fs;
use std::time::{Duration, SystemTime};

use codex_project_mover::surfaces::jsonl::{scan_jsonl_file, update_jsonl_file};
use tempfile::tempdir;

fn stamp_mtime(path: &std::path::Path) -> SystemTime {
    let mtime = SystemTime::UNIX_EPOCH + Duration::from_secs(1_600_000_000);
    fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(mtime)
        .unwrap();
    mtime
}

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

#[test]
fn preserves_mtime_when_updating() {
    let temp = tempdir().unwrap();
    let file = temp.path().join("thread.jsonl");
    fs::write(
        &file,
        r#"{"payload":{"cwd":"/old/project"}}"#.to_owned() + "\n",
    )
    .unwrap();
    let original_mtime = stamp_mtime(&file);

    let count = update_jsonl_file(&file, "/old/project", "/new/project").unwrap();

    assert_eq!(count, 1);
    assert!(fs::read_to_string(&file)
        .unwrap()
        .contains(r#""cwd":"/new/project""#));
    assert_eq!(
        fs::metadata(&file).unwrap().modified().unwrap(),
        original_mtime
    );
}

#[test]
fn leaves_files_without_matches_byte_identical() {
    let temp = tempdir().unwrap();
    let file = temp.path().join("thread.jsonl");
    let original =
        r#"{"payload":{"cwd":"/other/project"},"message":"mentions /old/project in text"}"#
            .to_owned()
            + "\n";
    fs::write(&file, &original).unwrap();
    let original_mtime = stamp_mtime(&file);

    let count = update_jsonl_file(&file, "/old/project", "/new/project").unwrap();

    assert_eq!(count, 0);
    assert_eq!(fs::read_to_string(&file).unwrap(), original);
    assert_eq!(
        fs::metadata(&file).unwrap().modified().unwrap(),
        original_mtime
    );
}
