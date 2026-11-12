#[test]
fn cache_snapshot_with_invalid_time_or_percent_is_rejected() {
    assert!(
        cosmic_applet_opencode_quota::cache::load_snapshot(Some(std::path::Path::new(
            "tests/invalid-cache.json"
        )))
        .is_none()
    );
}
