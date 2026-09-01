use cosmic_applet_opencode_quota::cache::{load_snapshot, save_snapshot};
use cosmic_applet_opencode_quota::quota::{QuotaSnapshot, QuotaWindow};
use tempfile::tempdir;

fn snap() -> QuotaSnapshot {
    QuotaSnapshot {
        rolling: QuotaWindow { percent_remaining: 42.0, resets_at: "r".into() },
        weekly: QuotaWindow { percent_remaining: 15.0, resets_at: "w".into() },
        monthly: QuotaWindow { percent_remaining: 80.0, resets_at: "m".into() },
        fetched_at_unix: 123,
    }
}

#[test]
fn roundtrip() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("nested").join("cache.json");
    save_snapshot(&path, &snap()).unwrap();
    let loaded = load_snapshot(Some(&path)).expect("load");
    assert_eq!(loaded.rolling.percent_remaining, 42.0);
    assert_eq!(loaded.weekly.percent_remaining, 15.0);
    assert_eq!(loaded.monthly.percent_remaining, 80.0);
    assert_eq!(loaded.fetched_at_unix, 123);
}

#[test]
fn load_missing_returns_none() {
    let dir = tempdir().unwrap();
    assert_eq!(load_snapshot(Some(&dir.path().join("nope.json"))), None);
}
