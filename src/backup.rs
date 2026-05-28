use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct BackupResult {
    pub backup_dir: PathBuf,
    pub manifest_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub version: u32,
    pub created_at_utc: String,
    pub old_path: String,
    pub new_path: String,
    pub created_new_project_path: Option<PathBuf>,
    pub entries: Vec<BackupEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupEntry {
    pub original_path: PathBuf,
    pub backup_path: PathBuf,
    pub sha256_before: String,
    pub byte_len: u64,
}

pub fn create_metadata_backup(
    backups_root: &Path,
    old_path: &str,
    new_path: &str,
    created_new_project_path: Option<PathBuf>,
    files: &[PathBuf],
) -> Result<BackupResult> {
    let id = unique_backup_id();
    let backup_dir = backups_root.join(id);
    let files_dir = backup_dir.join("files");
    fs::create_dir_all(&files_dir)
        .with_context(|| format!("create backup dir {}", files_dir.display()))?;

    let mut entries = Vec::new();
    for (index, original_path) in files.iter().enumerate() {
        let bytes = fs::read(original_path).with_context(|| {
            format!("read metadata file for backup {}", original_path.display())
        })?;
        let backup_path = files_dir.join(format!("{:04}.bak", index));
        fs::write(&backup_path, &bytes)
            .with_context(|| format!("write metadata backup {}", backup_path.display()))?;
        entries.push(BackupEntry {
            original_path: original_path.clone(),
            backup_path,
            sha256_before: sha256_hex(&bytes),
            byte_len: bytes.len() as u64,
        });
    }

    let manifest = BackupManifest {
        version: 1,
        created_at_utc: Utc::now().to_rfc3339(),
        old_path: old_path.to_string(),
        new_path: new_path.to_string(),
        created_new_project_path,
        entries,
    };
    let manifest_path = backup_dir.join("manifest.json");
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest)? + "\n",
    )
    .with_context(|| format!("write backup manifest {}", manifest_path.display()))?;

    Ok(BackupResult {
        backup_dir,
        manifest_path,
    })
}

pub fn restore_metadata_backup(manifest_path: &Path) -> Result<BackupManifest> {
    let manifest: BackupManifest = serde_json::from_slice(
        &fs::read(manifest_path)
            .with_context(|| format!("read backup manifest {}", manifest_path.display()))?,
    )
    .with_context(|| format!("parse backup manifest {}", manifest_path.display()))?;

    for entry in &manifest.entries {
        let bytes = fs::read(&entry.backup_path)
            .with_context(|| format!("read backup file {}", entry.backup_path.display()))?;
        anyhow::ensure!(
            sha256_hex(&bytes) == entry.sha256_before,
            "backup checksum mismatch for {}",
            entry.backup_path.display()
        );
        fs::write(&entry.original_path, bytes)
            .with_context(|| format!("restore metadata file {}", entry.original_path.display()))?;
    }
    Ok(manifest)
}

fn unique_backup_id() -> String {
    format!(
        "{}-{}",
        Utc::now().format("%Y%m%dT%H%M%SZ"),
        std::process::id()
    )
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
