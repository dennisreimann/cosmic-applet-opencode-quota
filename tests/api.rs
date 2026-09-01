use cosmic_applet_opencode_quota::api::fetch_quota_at;
use httpmock::prelude::*;

const VALID_BODY: &str = r#"{
  "usage": {
    "rolling": { "status": "ok", "percent": 58,  "resetsAt": "2026-09-01T20:00:00+02:00" },
    "weekly":  { "status": "ok", "percent": 85,  "resetsAt": "2026-09-06T00:00:00Z" },
    "monthly": { "status": "ok", "percent": 20,  "resetsAt": "2026-10-01T00:00:00+00:00" }
  }
}"#;

#[tokio::test]
async fn sends_auth_header_and_parses() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/usage")
            .header("authorization", "Bearer test-key");
        then.status(200).body(VALID_BODY);
    });
    let client = reqwest::Client::new();
    let url = format!("{}/usage", server.base_url());
    let snap = fetch_quota_at(&client, "test-key", &url).await.expect("ok");
    assert_eq!(snap.rolling.percent_remaining, 42.0);
    mock.assert_hits(1);
}

#[tokio::test]
async fn handles_401_without_retry() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET).path("/usage");
        then.status(401).body("unauthorized");
    });
    let client = reqwest::Client::new();
    let url = format!("{}/usage", server.base_url());
    let err = fetch_quota_at(&client, "bad", &url).await.unwrap_err();
    assert!(!err.retryable);
    assert!(err.message.contains("401"));
    mock.assert_hits(1);
}

#[tokio::test]
async fn retries_once_on_500() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET).path("/usage");
        then.status(500).body("boom");
    });
    let client = reqwest::Client::new();
    let url = format!("{}/usage", server.base_url());
    let err = fetch_quota_at(&client, "k", &url).await.unwrap_err();
    assert!(err.retryable);
    // 1 Versuch + 1 Retry = 2 Requests
    mock.assert_hits(2);
}
