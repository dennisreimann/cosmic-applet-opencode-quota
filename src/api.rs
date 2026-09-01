use crate::quota;
use reqwest::StatusCode;
use std::fmt;

pub const USAGE_URL: &str = "https://opencode.ai/zen/go/v1/usage";

#[derive(Debug)]
pub struct ApiError {
    pub message: String,
    pub retryable: bool,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "opencode go api: {}", self.message)
    }
}

impl std::error::Error for ApiError {}

fn is_retryable_status(code: StatusCode) -> bool {
    matches!(
        code,
        StatusCode::REQUEST_TIMEOUT
            | StatusCode::TOO_EARLY
            | StatusCode::TOO_MANY_REQUESTS
    ) || code.is_server_error()
}

const TIMEOUT_MS: u64 = 10_000;
const MAX_ATTEMPTS: usize = 2;

pub async fn fetch_quota(
    client: &reqwest::Client,
    api_key: &str,
) -> Result<quota::QuotaSnapshot, ApiError> {
    fetch_quota_at(client, api_key, USAGE_URL).await
}

pub async fn fetch_quota_at(
    client: &reqwest::Client,
    api_key: &str,
    url: &str,
) -> Result<quota::QuotaSnapshot, ApiError> {
    let mut attempt = 0;
    loop {
        let resp = client
            .get(url)
            .header(reqwest::header::AUTHORIZATION, format!("Bearer {api_key}"))
            .header(reqwest::header::ACCEPT, "application/json")
            .timeout(std::time::Duration::from_millis(TIMEOUT_MS))
            .send()
            .await
            .map_err(|e| ApiError {
                message: e.to_string(),
                retryable: true,
            })?;

        if resp.status().is_success() {
            let bytes = resp.bytes().await.map_err(|e| ApiError {
                message: e.to_string(),
                retryable: true,
            })?;
            return quota::parse_status(&bytes).map_err(|e| ApiError {
                message: e.to_string(),
                retryable: false,
            });
        }

        let retryable = is_retryable_status(resp.status());
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        let msg = format!("status {}: {}", status.as_u16(), text);
        if retryable && attempt + 1 < MAX_ATTEMPTS {
            attempt += 1;
            continue;
        }
        return Err(ApiError {
            message: msg,
            retryable,
        });
    }
}
