use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

const CRATES_IO_API: &str = "https://crates.io/api/v1/crates";
const CACHE_VERSION: u32 = 1;
const CACHE_MAX_AGE_DAYS: i64 = 7;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CrateInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub documentation: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub downloads: u64,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CrateMetadata {
    pub version: String,
    pub features: Vec<String>,
    pub all_features: Vec<String>,
    pub items: Vec<CrateItem>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CrateItem {
    pub name: String,
    pub kind: ItemKind,
    pub signature: Option<String>,
    pub docs: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ItemKind {
    Function,
    Struct,
    Enum,
    Trait,
    Type,
    Const,
    Static,
    Macro,
    Module,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CacheEntry {
    crate_info: CrateInfo,
    metadata: Option<CrateMetadata>,
    cached_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CacheIndex {
    version: u32,
    crates: HashMap<String, CacheEntry>,
}

pub struct CrateCache {
    cache_dir: PathBuf,
    index: Arc<RwLock<CacheIndex>>,
    client: reqwest::Client,
}

impl CrateCache {
    pub async fn new() -> Result<Self> {
        let cache_dir = Self::get_cache_dir()?;
        fs::create_dir_all(&cache_dir).await?;

        let index = Self::load_index(&cache_dir).await?;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("crab-lsp/0.1.0")
            .build()?;

        Ok(Self {
            cache_dir,
            index: Arc::new(RwLock::new(index)),
            client,
        })
    }

    pub fn new_sync() -> Self {
        let cache_dir = Self::get_cache_dir().unwrap_or_else(|_| PathBuf::from(".crab_cache"));
        let _ = std::fs::create_dir_all(&cache_dir);

        let index = Self::load_index_sync(&cache_dir);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("crab-lsp/0.1.0")
            .build()
            .unwrap_or_default();

        Self {
            cache_dir,
            index: Arc::new(RwLock::new(index)),
            client,
        }
    }

    fn load_index_sync(cache_dir: &Path) -> CacheIndex {
        let index_path = cache_dir.join("index.json");
        if index_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&index_path) {
                if let Ok(index) = serde_json::from_str::<CacheIndex>(&content) {
                    if index.version == CACHE_VERSION {
                        return index;
                    }
                }
            }
        }
        CacheIndex {
            version: CACHE_VERSION,
            crates: HashMap::new(),
        }
    }

    fn get_cache_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        Ok(PathBuf::from(home).join(".crab_cache").join("lsp-crates"))
    }

    async fn load_index(cache_dir: &Path) -> Result<CacheIndex> {
        let index_path = cache_dir.join("index.json");
        if index_path.exists() {
            let content = fs::read_to_string(&index_path).await?;
            let index: CacheIndex = serde_json::from_str(&content)?;
            if index.version == CACHE_VERSION {
                return Ok(index);
            }
        }
        Ok(CacheIndex {
            version: CACHE_VERSION,
            crates: HashMap::new(),
        })
    }

    async fn save_index(&self) -> Result<()> {
        let index_path = self.cache_dir.join("index.json");
        let index = self.index.read().await;
        let content = serde_json::to_string_pretty(&*index)?;
        drop(index);
        fs::write(&index_path, content).await?;
        Ok(())
    }

    pub async fn search_crates(&self, query: &str) -> Result<Vec<CrateInfo>> {
        let url = format!("{}/?q={}&per_page=20", CRATES_IO_API, query);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Ok(vec![]);
        }

        let data: serde_json::Value = response.json().await?;
        let mut results = vec![];

        if let Some(crates) = data.get("crates").and_then(|c| c.as_array()) {
            for c in crates {
                let info = CrateInfo {
                    name: c.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string(),
                    version: c.get("max_version")
                        .or_else(|| c.get("newest_version"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("0.1.0")
                        .to_string(),
                    description: c.get("description").and_then(|d| d.as_str()).map(|s| s.to_string()),
                    documentation: c.get("documentation").and_then(|d| d.as_str()).map(|s| s.to_string()),
                    homepage: c.get("homepage").and_then(|h| h.as_str()).map(|s| s.to_string()),
                    repository: c.get("repository").and_then(|r| r.as_str()).map(|s| s.to_string()),
                    downloads: c.get("downloads").and_then(|d| d.as_u64()).unwrap_or(0),
                    categories: vec![],
                    keywords: c.get("keywords")
                        .and_then(|k| k.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                        .unwrap_or_default(),
                    updated_at: Utc::now(),
                };
                results.push(info);
            }
        }

        Ok(results)
    }

    pub async fn get_crate_info(&self, name: &str) -> Result<Option<CrateInfo>> {
        {
            let index = self.index.read().await;
            if let Some(entry) = index.crates.get(name) {
                let age = Utc::now().signed_duration_since(entry.cached_at).num_days();
                if age < CACHE_MAX_AGE_DAYS {
                    return Ok(Some(entry.crate_info.clone()));
                }
            }
        }

        let url = format!("{}/{}", CRATES_IO_API, name);
        let response = self.client.get(&url).send().await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                let data: serde_json::Value = resp.json().await?;
                let c = data.get("crate");

                let info = CrateInfo {
                    name: name.to_string(),
                    version: c.and_then(|v| v.get("max_version"))
                        .or_else(|| c.and_then(|v| v.get("newest_version")))
                        .and_then(|v| v.as_str())
                        .unwrap_or("0.1.0")
                        .to_string(),
                    description: c.and_then(|v| v.get("description")).and_then(|d| d.as_str()).map(|s| s.to_string()),
                    documentation: c.and_then(|v| v.get("documentation")).and_then(|d| d.as_str()).map(|s| s.to_string()),
                    homepage: c.and_then(|v| v.get("homepage")).and_then(|h| h.as_str()).map(|s| s.to_string()),
                    repository: c.and_then(|v| v.get("repository")).and_then(|r| r.as_str()).map(|s| s.to_string()),
                    downloads: c.and_then(|v| v.get("downloads")).and_then(|d| d.as_u64()).unwrap_or(0),
                    categories: c.and_then(|v| v.get("categories"))
                        .and_then(|c| c.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                        .unwrap_or_default(),
                    keywords: c.and_then(|v| v.get("keywords"))
                        .and_then(|k| k.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                        .unwrap_or_default(),
                    updated_at: Utc::now(),
                };

                let entry = CacheEntry {
                    crate_info: info.clone(),
                    metadata: None,
                    cached_at: Utc::now(),
                };

                {
                    let mut index = self.index.write().await;
                    index.crates.insert(name.to_string(), entry);
                }
                self.save_index().await?;

                Ok(Some(info))
            }
            _ => {
                let index = self.index.read().await;
                Ok(index.crates.get(name).map(|e| e.crate_info.clone()))
            }
        }
    }

    pub async fn get_cached_crates(&self) -> Vec<String> {
        let index = self.index.read().await;
        index.crates.keys().cloned().collect()
    }

    pub async fn load_local_crate(&self, name: &str, path: &Path) -> Result<()> {
        let cargo_toml = path.join("Cargo.toml");
        if !cargo_toml.exists() {
            return Err(anyhow::anyhow!("No Cargo.toml found"));
        }

        let content = fs::read_to_string(&cargo_toml).await?;
        let manifest: toml::Value = toml::from_str(&content)?;

        let package = manifest.get("package").ok_or_else(|| anyhow::anyhow!("No [package] section"))?;
        let version = package.get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.1.0")
            .to_string();
        let description = package.get("description").and_then(|d| d.as_str()).map(|s| s.to_string());

        let info = CrateInfo {
            name: name.to_string(),
            version,
            description,
            documentation: None,
            homepage: None,
            repository: None,
            downloads: 0,
            categories: vec![],
            keywords: vec![],
            updated_at: Utc::now(),
        };

        let entry = CacheEntry {
            crate_info: info,
            metadata: None,
            cached_at: Utc::now(),
        };

        {
            let mut index = self.index.write().await;
            index.crates.insert(name.to_string(), entry);
        }
        self.save_index().await?;

        Ok(())
    }

    pub async fn warm_cache(&self, popular_crates: &[&str]) {
        for name in popular_crates {
            let _ = self.get_crate_info(name).await;
        }
    }

    pub async fn get_crate_items(&self, crate_name: &str) -> Vec<CrateItem> {
        let index = self.index.read().await;
        if let Some(entry) = index.crates.get(crate_name) {
            if let Some(ref metadata) = entry.metadata {
                return metadata.items.clone();
            }
        }
        vec![]
    }
}
