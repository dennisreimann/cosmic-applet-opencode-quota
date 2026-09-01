use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuotaWindow {
    pub percent_remaining: f64,
    pub resets_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuotaSnapshot {
    pub rolling: QuotaWindow,
    pub weekly: QuotaWindow,
    pub monthly: QuotaWindow,
    pub fetched_at_unix: i64,
}

impl QuotaSnapshot {
    pub fn overall_percent_remaining(&self) -> f64 {
        self.rolling
            .percent_remaining
            .min(self.weekly.percent_remaining)
            .min(self.monthly.percent_remaining)
    }
}

#[derive(Debug)]
pub enum QuotaError {
    Json(serde_json::Error),
    Contract(String),
}

impl fmt::Display for QuotaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuotaError::Json(e) => write!(f, "invalid json: {e}"),
            QuotaError::Contract(m) => write!(f, "invalid OpenCode Go API response: {m}"),
        }
    }
}

impl std::error::Error for QuotaError {}

#[derive(Deserialize)]
struct RawResponse {
    usage: RawUsage,
}

#[derive(Deserialize)]
struct RawUsage {
    rolling: RawWindow,
    weekly: RawWindow,
    monthly: RawWindow,
}

#[derive(Deserialize)]
struct RawWindow {
    status: String,
    percent: Option<f64>,
    #[serde(rename = "resetsAt")]
    resets_at: Option<String>,
}

fn normalize(name: &str, w: RawWindow) -> Result<QuotaWindow, QuotaError> {
    if w.status != "ok" {
        return Err(QuotaError::Contract(format!("{name} status is not ok")));
    }
    let percent = w
        .percent
        .ok_or_else(|| QuotaError::Contract(format!("{name} percent missing")))?;
    if !percent.is_finite() || !(0.0..=100.0).contains(&percent) {
        return Err(QuotaError::Contract(format!(
            "{name} percent must be finite from 0 to 100"
        )));
    }
    let resets_at = w
        .resets_at
        .ok_or_else(|| QuotaError::Contract(format!("{name} resetsAt missing")))?;
    if resets_at.trim().is_empty() {
        return Err(QuotaError::Contract(format!(
            "{name} resetsAt must not be empty"
        )));
    }
    Ok(QuotaWindow {
        percent_remaining: 100.0 - percent,
        resets_at,
    })
}

pub fn parse_status(raw: &[u8]) -> Result<QuotaSnapshot, QuotaError> {
    let parsed: RawResponse = serde_json::from_slice(raw).map_err(QuotaError::Json)?;
    let rolling = normalize("rolling", parsed.usage.rolling)?;
    let weekly = normalize("weekly", parsed.usage.weekly)?;
    let monthly = normalize("monthly", parsed.usage.monthly)?;
    let fetched_at_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    Ok(QuotaSnapshot {
        rolling,
        weekly,
        monthly,
        fetched_at_unix,
    })
}
