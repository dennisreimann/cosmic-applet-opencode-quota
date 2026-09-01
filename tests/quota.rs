use cosmic_applet_opencode_quota::quota::{parse_status, QuotaError};

const VALID: &str = r#"{
  "usage": {
    "rolling": { "status": "ok", "percent": 58,  "resetsAt": "2026-09-01T20:00:00+02:00" },
    "weekly":  { "status": "ok", "percent": 85,  "resetsAt": "2026-09-06T00:00:00Z" },
    "monthly": { "status": "ok", "percent": 20,  "resetsAt": "2026-10-01T00:00:00+00:00" }
  }
}"#;

#[test]
fn parses_valid_response() {
    let snap = parse_status(VALID.as_bytes()).expect("valid");
    assert_eq!(snap.rolling.percent_remaining, 42.0);
    assert_eq!(snap.weekly.percent_remaining, 15.0);
    assert_eq!(snap.monthly.percent_remaining, 80.0);
    assert_eq!(snap.rolling.resets_at, "2026-09-01T20:00:00+02:00");
    assert!(snap.fetched_at_unix > 0);
}

#[test]
fn overall_is_min() {
    let snap = parse_status(VALID.as_bytes()).unwrap();
    assert_eq!(snap.overall_percent_remaining(), 15.0);
}

#[test]
fn missing_window_or_usage_is_error() {
    // usage fehlt ganz
    let raw2 = br#"{"weekly": {}}"#;
    assert!(parse_status(raw2).is_err());
    // weekly fehlt innerhalb von usage
    let raw = br#"{"usage": { "rolling": { "status":"ok","percent":10,"resetsAt":"2026-09-01T20:00:00+02:00" } } }"#;
    assert!(parse_status(raw).is_err());
}

#[test]
fn bad_status_is_contract_error() {
    let raw = br#"{"usage": {
        "rolling": { "status":"error" },
        "weekly":  { "status":"ok","percent":10,"resetsAt":"2026-09-01T20:00:00+02:00" },
        "monthly": { "status":"ok","percent":10,"resetsAt":"2026-09-01T20:00:00+02:00" }
    } }"#;
    assert!(matches!(parse_status(raw), Err(QuotaError::Contract(_))));
}

#[test]
fn percent_out_of_range_is_contract_error() {
    let raw = br#"{"usage": {
        "rolling": { "status":"ok","percent":150,"resetsAt":"2026-09-01T20:00:00+02:00" },
        "weekly":  { "status":"ok","percent":10,"resetsAt":"2026-09-01T20:00:00+02:00" },
        "monthly": { "status":"ok","percent":10,"resetsAt":"2026-09-01T20:00:00+02:00" }
    } }"#;
    assert!(matches!(parse_status(raw), Err(QuotaError::Contract(_))));
}

#[test]
fn invalid_json_is_json_error() {
    assert!(matches!(parse_status(b"not json"), Err(QuotaError::Json(_))));
}
