//! Backup and restore for profiles, configs, and runtime files.
//! Supports local zip archives and WebDAV remote storage.

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::time::SystemTime;

/// Backup manager for local and remote (WebDAV) backups.
pub struct BackupManager {
    data_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct BackupEntry {
    pub name: String,
    pub timestamp: u64,
    pub size: u64,
}

impl BackupManager {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    /// Create a local zip backup of profiles and configs.
    pub async fn create_local_backup(&self) -> Result<BackupEntry> {
        let ts = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let backup_dir = self.data_dir.join("backups");
        std::fs::create_dir_all(&backup_dir)?;

        let name = format!("zeroclash-backup-{ts}.zip");
        let path = backup_dir.join(&name);

        let files_to_backup = self.collect_backup_files()?;

        let file = std::fs::File::create(&path)?;
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        for (archive_name, file_path) in &files_to_backup {
            if file_path.exists() {
                let content = std::fs::read(file_path)
                    .with_context(|| format!("read {file_path:?}"))?;
                zip.start_file(archive_name, options)?;
                std::io::Write::write_all(&mut zip, &content)?;
            }
        }

        let file = zip.finish()?;
        let size = file.metadata()?.len();

        Ok(BackupEntry { name, timestamp: ts, size })
    }

    /// List available local backups.
    pub fn list_local_backups(&self) -> Result<Vec<BackupEntry>> {
        let backup_dir = self.data_dir.join("backups");
        if !backup_dir.exists() {
            return Ok(vec![]);
        }

        let mut entries = Vec::new();
        for entry in std::fs::read_dir(&backup_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "zip") {
                let meta = entry.metadata()?;
                let name = entry.file_name().to_string_lossy().to_string();
                let ts = meta.modified()
                    .ok()
                    .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                entries.push(BackupEntry { name, timestamp: ts, size: meta.len() });
            }
        }
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(entries)
    }

    /// Restore from a local backup by name.
    pub async fn restore_local_backup(&self, name: &str) -> Result<()> {
        let path = self.data_dir.join("backups").join(name);
        if !path.exists() {
            anyhow::bail!("Backup {name} not found");
        }

        let file = std::fs::File::open(&path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            let out_path = self.data_dir.join(entry.name());
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut out_file = std::fs::File::create(&out_path)?;
            std::io::copy(&mut entry, &mut out_file)?;
        }

        Ok(())
    }

    /// Upload a backup to WebDAV server.
    pub async fn upload_to_webdav(
        &self,
        name: &str,
        webdav_url: &str,
        username: &str,
        password: &str,
    ) -> Result<()> {
        let path = self.data_dir.join("backups").join(name);
        let data = tokio::fs::read(&path)
            .await
            .with_context(|| format!("read backup {name}"))?;

        let client = reqwest::Client::new();
        let url = format!("{}/{}", webdav_url.trim_end_matches('/'), name);
        let resp = client
            .put(&url)
            .basic_auth(username, Some(password))
            .body(data)
            .send()
            .await
            .with_context(|| format!("upload to {url}"))?;

        if !resp.status().is_success() {
            anyhow::bail!("WebDAV upload failed: {}", resp.status());
        }
        Ok(())
    }

    /// Download a backup from WebDAV server.
    pub async fn download_from_webdav(
        &self,
        name: &str,
        webdav_url: &str,
        username: &str,
        password: &str,
    ) -> Result<()> {
        let client = reqwest::Client::new();
        let url = format!("{}/{}", webdav_url.trim_end_matches('/'), name);
        let resp = client
            .get(&url)
            .basic_auth(username, Some(password))
            .send()
            .await
            .with_context(|| format!("download from {url}"))?;

        if !resp.status().is_success() {
            anyhow::bail!("WebDAV download failed: {}", resp.status());
        }

        let data = resp.bytes().await?;
        let backup_dir = self.data_dir.join("backups");
        std::fs::create_dir_all(&backup_dir)?;

        tokio::fs::write(backup_dir.join(name), &data)
            .await
            .context("save downloaded backup")?;

        Ok(())
    }

    fn collect_backup_files(&self) -> Result<Vec<(String, PathBuf)>> {
        let mut files = Vec::new();

        // Profiles YAML
        let profiles = self.data_dir.join("profiles.yaml");
        if profiles.exists() {
            files.push(("profiles.yaml".into(), profiles));
        }

        // All profile data files
        let profile_dir = self.data_dir.to_path_buf();
        if let Ok(entries) = std::fs::read_dir(&profile_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".yaml") && name != "profiles.yaml" {
                    files.push((name.clone(), entry.path()));
                }
                if name.ends_with(".js") {
                    files.push((name, entry.path()));
                }
            }
        }

        Ok(files)
    }
}
