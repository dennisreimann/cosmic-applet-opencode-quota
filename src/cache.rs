use crate::quota::QuotaSnapshot;
use anyhow::Context;
use std::fs;
use std::path::Path;

pub fn load_snapshot(path: Option<&Path>) -> Option<QuotaSnapshot> {
    let path = path?;
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

pub fn save_snapshot(path: &Path, snap: &QuotaSnapshot) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("create cache dir")?;
    }
    let json = serde_json::to_vec_pretty(snap).context("serialize cache")?;
    fs::write(path, json).context("write cache")?;
    Ok(())
}
