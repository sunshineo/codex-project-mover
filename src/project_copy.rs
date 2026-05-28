use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

pub fn copy_project_tree(old: &Path, new: &Path) -> Result<()> {
    if let Some(parent) = new.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create new project parent {}", parent.display()))?;
    }

    for entry in WalkDir::new(old).follow_links(false) {
        let entry = entry?;
        let source = entry.path();
        let relative = source.strip_prefix(old)?;
        let target = new.join(relative);
        let file_type = entry.file_type();

        if relative.as_os_str().is_empty() || file_type.is_dir() {
            fs::create_dir_all(&target)?;
        } else if file_type.is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(source, &target)
                .with_context(|| format!("copy {} to {}", source.display(), target.display()))?;
            fs::set_permissions(&target, fs::metadata(source)?.permissions())?;
        } else if file_type.is_symlink() {
            let link_target = fs::read_link(source)?;
            #[cfg(unix)]
            std::os::unix::fs::symlink(link_target, &target)?;
        }
    }

    Ok(())
}

pub fn verify_project_tree(old: &Path, new: &Path) -> Result<()> {
    let old_entries = tree_fingerprints(old)?;
    let new_entries = tree_fingerprints(new)?;
    anyhow::ensure!(
        old_entries == new_entries,
        "copied project tree does not match source"
    );
    Ok(())
}

fn tree_fingerprints(root: &Path) -> Result<Vec<(PathBuf, String)>> {
    let mut entries = Vec::new();
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(root)?.to_path_buf();
        if relative.as_os_str().is_empty() {
            continue;
        }

        let kind_hash = if entry.file_type().is_dir() {
            "dir".to_string()
        } else if entry.file_type().is_symlink() {
            format!("symlink:{}", fs::read_link(path)?.display())
        } else {
            format!("file:{}", sha256_file(path)?)
        };
        entries.push((relative, kind_hash));
    }
    entries.sort();
    Ok(entries)
}

fn sha256_file(path: &Path) -> Result<String> {
    let bytes = fs::read(path)?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
