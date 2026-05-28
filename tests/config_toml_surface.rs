use std::fs;

use codex_project_mover::surfaces::config_toml::{scan_config_toml, update_config_toml};
use tempfile::tempdir;

#[test]
fn updates_exact_project_table_key_per_path_key_and_exact_string_values() {
    let temp = tempdir().unwrap();
    let file = temp.path().join("config.toml");
    fs::write(
        &file,
        r#"
[projects."/old/project"]
trust_level = "trusted"

[desktop]
open_target = "/old/project"
message = "do not edit /old/project inside text"

[desktop.open-in-target-preferences.perPath]
"/old/project" = "vscode"
"/old/project/subdir" = "iterm2"
"#,
    )
    .unwrap();

    let matches = scan_config_toml(&file, "/old/project", "/new/project").unwrap();
    assert_eq!(matches.len(), 3);

    let changed = update_config_toml(&file, "/old/project", "/new/project").unwrap();
    let updated = fs::read_to_string(&file).unwrap();

    assert_eq!(changed, 3);
    assert!(updated.contains(r#"[projects."/new/project"]"#));
    assert!(updated.contains(r#"open_target = "/new/project""#));
    assert!(updated.contains(r#""/new/project" = "vscode""#));
    assert!(updated.contains(r#"message = "do not edit /old/project inside text""#));
}
