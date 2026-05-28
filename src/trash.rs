use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

pub fn move_to_trash(path: &Path) -> Result<()> {
    if let Ok(test_trash_dir) = std::env::var("CODEX_PROJECT_MOVER_TEST_TRASH_DIR") {
        let file_name = path
            .file_name()
            .context("project path must have a final path component")?;
        let target = Path::new(&test_trash_dir).join(file_name);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(path, &target)
            .with_context(|| format!("move project folder to test trash: {}", target.display()))?;
        return Ok(());
    }

    trash::delete(path).with_context(|| format!("move project folder to Trash: {}", path.display()))
}
