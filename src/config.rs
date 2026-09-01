use dirs;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
struct AuthEntry {
    #[serde(rename = "type")]
    kind: String,
    key: Option<String>,
}

pub fn auth_json_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("opencode").join("auth.json"))
}

pub fn default_cache_path() -> Option<PathBuf> {
    dirs::cache_dir().map(|d| d.join("opencode-quota").join("cache.json"))
}

pub fn resolve_api_key(auth_file: Option<&Path>) -> Option<String> {
    if let Some(path) = auth_file {
        if let Ok(text) = fs::read_to_string(path) {
            if let Ok(map) = serde_json::from_str::<HashMap<String, AuthEntry>>(&text) {
                if let Some(entry) = map.get("opencode-go") {
                    if entry.kind == "api" {
                        if let Some(key) = &entry.key {
                            if !key.is_empty() {
                                return Some(key.clone());
                            }
                        }
                    }
                }
            }
        }
    }
    std::env::var("OPENCODE_API_KEY").ok().filter(|k| !k.is_empty())
}
