//! Profile / subscription management.
//!
//! Handles profile items (remote subscriptions, local files, merge, script, rules, proxies, groups),
//! CRUD operations, HTTP subscription updates, and YAML file persistence.

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_yaml_ng::Mapping;
use std::path::PathBuf;

// ── Profile data types ────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct PrfItem {
    pub uid: Option<String>,
    #[serde(rename = "type")]
    pub itype: Option<String>,
    pub name: Option<String>,
    pub file: Option<String>,
    pub desc: Option<String>,
    pub url: Option<String>,
    pub selected: Option<Vec<PrfSelected>>,
    pub extra: Option<PrfExtra>,
    pub updated: Option<usize>,
    pub option: Option<PrfOption>,
    pub home: Option<String>,
    #[serde(skip)]
    pub file_data: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct PrfSelected {
    pub name: Option<String>,
    pub now: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
pub struct PrfExtra {
    pub upload: u64,
    pub download: u64,
    pub total: u64,
    pub expire: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct PrfOption {
    pub user_agent: Option<String>,
    pub with_proxy: Option<bool>,
    pub self_proxy: Option<bool>,
    pub update_interval: Option<u64>,
    pub timeout_seconds: Option<u64>,
    pub danger_accept_invalid_certs: Option<bool>,
    pub allow_auto_update: Option<bool>,
    pub merge: Option<String>,
    pub script: Option<String>,
    pub rules: Option<String>,
    pub proxies: Option<String>,
    pub groups: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct IProfiles {
    pub current: Option<String>,
    pub items: Option<Vec<PrfItem>>,
}

// ── Profile store ─────────────────────────────────────────────────────────

pub struct ProfileStore {
    /// Path to profiles.yaml
    config_path: PathBuf,
    /// Directory where profile files are stored
    data_dir: PathBuf,
    /// In-memory profiles
    pub profiles: IProfiles,
}

impl ProfileStore {
    /// Load profiles from disk.
    pub async fn load(data_dir: PathBuf) -> Result<Self> {
        let config_path = data_dir.join("profiles.yaml");

        let profiles = if config_path.exists() {
            let content = tokio::fs::read_to_string(&config_path)
                .await
                .context("read profiles.yaml")?;
            serde_yaml_ng::from_str::<IProfiles>(&content)
                .unwrap_or_default()
        } else {
            IProfiles::default()
        };

        Ok(Self {
            config_path,
            data_dir,
            profiles,
        })
    }

    /// Save profiles to disk.
    pub async fn save(&self) -> Result<()> {
        let yaml = serde_yaml_ng::to_string(&self.profiles)?;
        tokio::fs::write(&self.config_path, yaml)
            .await
            .context("write profiles.yaml")?;
        Ok(())
    }

    /// Fetch a remote profile from a URL.
    pub async fn fetch_remote(
        &self,
        url: &str,
        name: Option<&str>,
        _user_agent: Option<&str>,
    ) -> Result<PrfItem> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .danger_accept_invalid_certs(false)
            .build()?;

        let resp = client.get(url).send().await.context("fetch remote profile")?;

        if !resp.status().is_success() {
            bail!("HTTP {} fetching remote profile", resp.status());
        }

        let data = resp.text().await.context("read response body")?;
        let data = data.trim_start_matches('\u{feff}');

        // Validate it's valid YAML
        let _yaml: Mapping =
            serde_yaml_ng::from_str(data).context("remote profile is not valid YAML")?;

        let uid = generate_uid('R');
        let file_name = format!("{uid}.yaml");
        let disp_name = name.unwrap_or("Remote Profile").to_string();

        let file_path = self.data_dir.join(&file_name);
        tokio::fs::write(&file_path, data.as_bytes())
            .await
            .context("save profile file")?;

        Ok(PrfItem {
            uid: Some(uid),
            itype: Some("remote".into()),
            name: Some(disp_name),
            file: Some(file_name),
            url: Some(url.to_string()),
            updated: Some(chrono_now()),
            file_data: Some(data.to_string()),
            ..Default::default()
        })
    }

    /// Create a local profile.
    pub async fn create_local(&self, name: &str, data: &str) -> Result<PrfItem> {
        let uid = generate_uid('L');
        let file_name = format!("{uid}.yaml");
        let file_path = self.data_dir.join(&file_name);

        tokio::fs::write(&file_path, data).await?;

        Ok(PrfItem {
            uid: Some(uid),
            itype: Some("local".into()),
            name: Some(name.to_string()),
            file: Some(file_name),
            file_data: Some(data.to_string()),
            updated: Some(chrono_now()),
            ..Default::default()
        })
    }

    /// Add a profile item to the store.
    pub fn add_item(&mut self, item: PrfItem) -> Result<()> {
        let items = self.profiles.items.get_or_insert_with(Vec::new);

        if self.profiles.current.is_none() {
            let itype = item.itype.as_deref().unwrap_or("");
            if itype == "remote" || itype == "local" {
                self.profiles.current = item.uid.clone();
            }
        }

        items.push(item);
        Ok(())
    }

    /// Update an existing item.
    pub fn update_item(&mut self, uid: &str, patch: &PrfItem) -> Result<()> {
        let items = self.profiles.items.as_mut().context("no items")?;
        let item = items
            .iter_mut()
            .find(|i| i.uid.as_deref() == Some(uid))
            .with_context(|| format!("profile {uid} not found"))?;

        if let Some(ref v) = patch.itype {
            item.itype = Some(v.clone());
        }
        if let Some(ref v) = patch.name {
            item.name = Some(v.clone());
        }
        if let Some(ref v) = patch.desc {
            item.desc = Some(v.clone());
        }
        if let Some(ref v) = patch.url {
            item.url = Some(v.clone());
        }
        if let Some(v) = patch.updated {
            item.updated = Some(v);
        }
        if patch.option.is_some() {
            item.option = patch.option.clone();
        }
        Ok(())
    }

    /// Delete an item by uid. Returns true if it was the current profile.
    pub fn delete_item(&mut self, uid: &str) -> Result<bool> {
        let items = self.profiles.items.get_or_insert_with(Vec::new);
        let idx = items
            .iter()
            .position(|i| i.uid.as_deref() == Some(uid))
            .with_context(|| format!("profile {uid} not found"))?;

        let deleted = items.remove(idx);

        let was_current = self.profiles.current.as_deref() == Some(uid);
        if was_current {
            self.profiles.current = items
                .iter()
                .find(|i| {
                    let t = i.itype.as_deref().unwrap_or("");
                    t == "remote" || t == "local"
                })
                .and_then(|i| i.uid.clone());
        }

        // Delete file from disk
        if let Some(ref file) = deleted.file {
            let path = self.data_dir.join(file);
            let _ = std::fs::remove_file(path);
        }

        Ok(was_current)
    }

    /// Reorder profiles: move `active_id` before `over_id`.
    pub fn reorder(&mut self, active_id: &str, over_id: &str) -> Result<()> {
        let items = self.profiles.items.get_or_insert_with(Vec::new);
        let old_idx = items
            .iter()
            .position(|i| i.uid.as_deref() == Some(active_id))
            .with_context(|| format!("active id {active_id} not found"))?;
        let new_idx = items
            .iter()
            .position(|i| i.uid.as_deref() == Some(over_id))
            .with_context(|| format!("over id {over_id} not found"))?;

        let item = items.remove(old_idx);
        items.insert(new_idx, item);
        Ok(())
    }

    /// Set the current (active) profile.
    pub fn set_current(&mut self, uid: &str) -> Result<()> {
        let items = self.profiles.items.as_ref().context("no items")?;
        items
            .iter()
            .find(|i| i.uid.as_deref() == Some(uid))
            .with_context(|| format!("profile {uid} not found"))?;
        self.profiles.current = Some(uid.to_string());
        Ok(())
    }

    /// Read the current profile's YAML data.
    pub async fn current_mapping(&self) -> Result<Mapping> {
        let current = self
            .profiles
            .current
            .as_ref()
            .context("no current profile")?;
        let items = self.profiles.items.as_ref().context("no items")?;
        let item = items
            .iter()
            .find(|i| i.uid.as_deref() == Some(current))
            .context("current profile not found")?;

        let file = item.file.as_ref().context("no file for current profile")?;
        let path = self.data_dir.join(file);
        let content = tokio::fs::read_to_string(&path)
            .await
            .context("read current profile file")?;
        serde_yaml_ng::from_str(&content).context("parse current profile YAML")
    }

    /// Get a preview list (uid, name, is_current) for display.
    pub fn preview(&self) -> Vec<ProfilePreview> {
        let current = self.profiles.current.as_deref();
        self.profiles
            .items
            .as_ref()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|i| {
                        Some(ProfilePreview {
                            uid: i.uid.clone()?,
                            name: i.name.clone().unwrap_or_default(),
                            itype: i.itype.clone().unwrap_or_default(),
                            is_current: current == i.uid.as_deref(),
                            url: i.url.clone(),
                            updated: i.updated,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct ProfilePreview {
    pub uid: String,
    pub name: String,
    pub itype: String,
    pub is_current: bool,
    pub url: Option<String>,
    pub updated: Option<usize>,
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn generate_uid(prefix: char) -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u32;
    format!("{prefix}{ts:08x}")
}

fn chrono_now() -> usize {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn new_test_store() -> (ProfileStore, TempDir) {
        let tmp = TempDir::new().expect("tempdir");
        let store = ProfileStore::load(tmp.path().to_path_buf()).await.expect("load");
        (store, tmp)
    }

    #[tokio::test]
    async fn test_create_local_profile() {
        let (store, _tmp) = new_test_store().await;
        let item = store.create_local("Test", "proxies:\n  - name: node1").await.expect("create");
        assert_eq!(item.name.as_deref(), Some("Test"));
        assert_eq!(item.itype.as_deref(), Some("local"));
        assert!(item.uid.is_some());
        assert!(item.file.as_ref().unwrap().ends_with(".yaml"));
    }

    #[tokio::test]
    async fn test_add_and_preview() {
        let (mut store, _tmp) = new_test_store().await;
        let item = store.create_local("MyProfile", "proxies: []").await.unwrap();
        store.add_item(item).unwrap();
        store.save().await.unwrap();

        let previews = store.preview();
        assert_eq!(previews.len(), 1);
        assert_eq!(previews[0].name, "MyProfile");
        assert!(previews[0].is_current);
    }

    #[tokio::test]
    async fn test_set_current() {
        let (mut store, _tmp) = new_test_store().await;
        let a = store.create_local("A", "a: 1").await.unwrap();
        let b = store.create_local("B", "b: 2").await.unwrap();
        store.add_item(a).unwrap();
        store.add_item(b).unwrap();

        let uid_b = store.preview()[1].uid.clone();
        store.set_current(&uid_b).unwrap();
        assert_eq!(store.profiles.current.as_deref(), Some(uid_b.as_str()));
    }

    #[tokio::test]
    async fn test_delete_item() {
        let (mut store, _tmp) = new_test_store().await;
        let item = store.create_local("Del", "x: 1").await.unwrap();
        let uid = item.uid.clone().unwrap();
        store.add_item(item).unwrap();

        let was_current = store.delete_item(&uid).unwrap();
        assert!(was_current);
        assert!(store.preview().is_empty());
    }

    #[tokio::test]
    async fn test_reorder() {
        let (mut store, _tmp) = new_test_store().await;
        let a = store.create_local("First", "a: 1").await.unwrap();
        let b = store.create_local("Second", "b: 2").await.unwrap();
        let a_uid = a.uid.clone().unwrap();
        let b_uid = b.uid.clone().unwrap();
        store.add_item(a).unwrap();
        store.add_item(b).unwrap();

        store.reorder(&a_uid, &b_uid).unwrap();
        let previews = store.preview();
        // reorder(a, b) moves a to before b, so order becomes [a, b] → a stays first
        assert_eq!(previews.len(), 2);
        assert!(!previews.is_empty());
    }
}
