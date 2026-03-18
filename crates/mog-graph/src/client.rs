use mog_core::error::MogError;
use reqwest::{Client, Method, Response, StatusCode};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, warn};

use crate::cloud::Cloud;
use crate::pagination;
use crate::retry::{self, RetryConfig};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Threshold at which a warning is emitted for large collections during pagination.
const COLLECTION_SIZE_WARNING_THRESHOLD: usize = 1000;

/// Options for a Graph request
#[derive(Debug, Default, Clone)]
pub struct RequestOptions {
    pub query_params: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: Option<Value>,
    pub body_bytes: Option<Vec<u8>>,
    pub raw_url: Option<String>,
}

/// Trace event for --trace mode
#[derive(Debug, serde::Serialize)]
pub struct TraceEvent {
    pub ts: String,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub duration_ms: u64,
    pub client_request_id: String,
    pub request_id: String,
    pub throttled: bool,
}

/// The Graph HTTP client with retry, throttling, pagination, and correlation
pub struct GraphClient {
    http: Client,
    base_url: String,
    api_version: String,
    access_token: String,
    retry_config: RetryConfig,
    max_results: u32,
    max_all_results: u32,
    upload_timeout: Duration,
    trace: bool,
}

impl GraphClient {
    pub fn new(
        access_token: String,
        cloud: Cloud,
        api_version: &str,
        max_retries: u32,
        max_results: u32,
        max_all_results: u32,
        timeout_seconds: u64,
        connect_timeout_seconds: u64,
        upload_timeout_seconds: u64,
        trace: bool,
    ) -> Result<Self, MogError> {
        let http = Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .connect_timeout(Duration::from_secs(connect_timeout_seconds))
            .pool_max_idle_per_host(10)
            .build()
            .map_err(|e| MogError::Network(format!("Failed to create HTTP client: {}", e)))?;

        let endpoints = cloud.endpoints();

        Ok(Self {
            http,
            base_url: endpoints.graph.to_string(),
            api_version: api_version.to_string(),
            access_token,
            retry_config: RetryConfig {
                max_retries,
                ..Default::default()
            },
            max_results,
            max_all_results,
            upload_timeout: Duration::from_secs(upload_timeout_seconds),
            trace,
        })
    }

    /// Placeholder constructor for when no auth is available yet
    pub fn unauthenticated(cloud: Cloud, api_version: &str) -> Result<Self, MogError> {
        Self::new(
            String::new(),
            cloud,
            api_version,
            3,
            100,
            10_000,
            30,
            10,
            300,
            false,
        )
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn api_version(&self) -> &str {
        &self.api_version
    }

    fn build_url(&self, path: &str) -> String {
        if path.starts_with("http://") || path.starts_with("https://") {
            return path.to_string();
        }
        let path = path.trim_start_matches('/');
        format!("{}/{}/{}", self.base_url, self.api_version, path)
    }

    fn user_agent() -> String {
        format!("mog/{} ({})", VERSION, std::env::consts::OS)
    }

    /// Execute a single Graph request with retry logic
    pub async fn request(
        &self,
        method: Method,
        path: &str,
        options: &RequestOptions,
    ) -> Result<Value, MogError> {
        let url = if let Some(ref raw) = options.raw_url {
            raw.clone()
        } else {
            self.build_url(path)
        };

        let mut attempt = 0;
        loop {
            let client_request_id = uuid::Uuid::new_v4().to_string();
            let start = std::time::Instant::now();

            let mut req = self
                .http
                .request(method.clone(), &url)
                .header("User-Agent", Self::user_agent())
                .header("client-request-id", &client_request_id);

            if !self.access_token.is_empty() {
                req = req.header("Authorization", format!("Bearer {}", self.access_token));
            }

            // Add query params
            if !options.query_params.is_empty() {
                let pairs: Vec<(&str, &str)> = options
                    .query_params
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_str()))
                    .collect();
                req = req.query(&pairs);
            }

            // Add custom headers
            for (k, v) in &options.headers {
                req = req.header(k.as_str(), v.as_str());
            }

            // Add body
            if let Some(ref body) = options.body {
                req = req.header("Content-Type", "application/json").json(body);
            } else if let Some(ref bytes) = options.body_bytes {
                req = req
                    .header("Content-Type", "application/octet-stream")
                    .body(bytes.clone());
            }

            let result = req.send().await;

            match result {
                Ok(response) => {
                    let status = response.status();
                    let duration = start.elapsed();

                    // Extract response headers before consuming body
                    let request_id = response
                        .headers()
                        .get("request-id")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("")
                        .to_string();

                    let retry_after = response
                        .headers()
                        .get("Retry-After")
                        .and_then(|v| v.to_str().ok())
                        .and_then(RetryConfig::parse_retry_after);

                    // Check for Sunset header (deprecation)
                    if let Some(sunset) = response.headers().get("Sunset") {
                        if let Ok(date) = sunset.to_str() {
                            eprintln!(
                                "Warning: This API endpoint is deprecated. Sunset date: {}",
                                date
                            );
                        }
                    }

                    // Trace event
                    if self.trace {
                        let event = TraceEvent {
                            ts: chrono::Utc::now().to_rfc3339(),
                            method: method.to_string(),
                            path: path.to_string(),
                            status: status.as_u16(),
                            duration_ms: duration.as_millis() as u64,
                            client_request_id: client_request_id.clone(),
                            request_id: request_id.clone(),
                            throttled: status == StatusCode::TOO_MANY_REQUESTS,
                        };
                        if let Ok(json) = serde_json::to_string(&event) {
                            eprintln!("{}", json);
                        }
                    }

                    debug!(
                        method = %method,
                        path = %path,
                        status = status.as_u16(),
                        duration_ms = duration.as_millis() as u64,
                        client_request_id = %client_request_id,
                        request_id = %request_id,
                        "Graph request"
                    );

                    match status {
                        s if s.is_success() => {
                            if status == StatusCode::NO_CONTENT {
                                return Ok(Value::Null);
                            }
                            let body = response.text().await.map_err(|e| {
                                MogError::Network(format!("Failed to read response: {}", e))
                            })?;
                            if body.is_empty() {
                                return Ok(Value::Null);
                            }
                            let value: Value =
                                serde_json::from_str(&body).unwrap_or(Value::String(body));
                            return Ok(value);
                        }
                        StatusCode::UNAUTHORIZED => {
                            return Err(MogError::Auth(format!(
                                "Authentication failed (401). client-request-id: {}",
                                client_request_id
                            )));
                        }
                        StatusCode::FORBIDDEN => {
                            let body = response.text().await.unwrap_or_default();
                            return Err(MogError::Authz(format!(
                                "Insufficient permissions (403): {}",
                                extract_error_message(&body)
                            )));
                        }
                        StatusCode::NOT_FOUND => {
                            let body = response.text().await.unwrap_or_default();
                            return Err(MogError::NotFound(format!(
                                "Resource not found (404): {}",
                                extract_error_message(&body)
                            )));
                        }
                        StatusCode::CONFLICT => {
                            let body = response.text().await.unwrap_or_default();
                            return Err(MogError::Conflict(format!(
                                "Conflict (409): {}",
                                extract_error_message(&body)
                            )));
                        }
                        StatusCode::TOO_MANY_REQUESTS => {
                            if attempt < self.retry_config.max_retries {
                                let delay = retry_after.unwrap_or_else(|| {
                                    self.retry_config.delay_for_attempt(attempt)
                                });
                                warn!(
                                    "Rate limited (429). Retrying in {:?} (attempt {}/{})",
                                    delay,
                                    attempt + 1,
                                    self.retry_config.max_retries
                                );
                                tokio::time::sleep(delay).await;
                                attempt += 1;
                                continue;
                            }
                            return Err(MogError::RateLimited(format!(
                                "Rate limited after {} retries. client-request-id: {}",
                                self.retry_config.max_retries, client_request_id
                            )));
                        }
                        s if retry::is_retryable_status(s.as_u16()) => {
                            if attempt < self.retry_config.max_retries {
                                let delay = retry_after.unwrap_or_else(|| {
                                    self.retry_config.delay_for_attempt(attempt)
                                });
                                warn!(
                                    "Retryable error ({}). Retrying in {:?} (attempt {}/{})",
                                    s.as_u16(),
                                    delay,
                                    attempt + 1,
                                    self.retry_config.max_retries
                                );
                                tokio::time::sleep(delay).await;
                                attempt += 1;
                                continue;
                            }
                            let body = response.text().await.unwrap_or_default();
                            return Err(MogError::General(format!(
                                "Server error ({}) after {} retries: {}",
                                s.as_u16(),
                                self.retry_config.max_retries,
                                extract_error_message(&body)
                            )));
                        }
                        _ => {
                            let body = response.text().await.unwrap_or_default();
                            return Err(MogError::General(format!(
                                "HTTP {} — {}",
                                status.as_u16(),
                                extract_error_message(&body)
                            )));
                        }
                    }
                }
                Err(e) => {
                    if e.is_connect() || e.is_timeout() {
                        if attempt < self.retry_config.max_retries {
                            let delay = self.retry_config.delay_for_attempt(attempt);
                            warn!(
                                "Network error: {}. Retrying in {:?} (attempt {}/{})",
                                e,
                                delay,
                                attempt + 1,
                                self.retry_config.max_retries
                            );
                            tokio::time::sleep(delay).await;
                            attempt += 1;
                            continue;
                        }
                        return Err(MogError::Network(format!(
                            "Cannot reach Microsoft Graph after {} retries. Check network connectivity. Error: {}",
                            self.retry_config.max_retries, e
                        )));
                    }
                    return Err(MogError::Network(format!("HTTP request failed: {}", e)));
                }
            }
        }
    }

    /// Execute a GET request and return the raw response bytes (for file downloads)
    pub async fn request_bytes(
        &self,
        method: Method,
        path: &str,
        options: &RequestOptions,
    ) -> Result<Vec<u8>, MogError> {
        let url = if let Some(ref raw) = options.raw_url {
            raw.clone()
        } else {
            self.build_url(path)
        };

        let client_request_id = uuid::Uuid::new_v4().to_string();

        let mut req = self
            .http
            .request(method, &url)
            .header("User-Agent", Self::user_agent())
            .header("client-request-id", &client_request_id);

        if !self.access_token.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.access_token));
        }

        for (k, v) in &options.headers {
            req = req.header(k.as_str(), v.as_str());
        }

        let response = req
            .send()
            .await
            .map_err(|e| MogError::Network(format!("Request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(Self::status_to_error(status, &body));
        }

        response
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| MogError::Network(format!("Failed to read response bytes: {}", e)))
    }

    /// Upload bytes to a URL (for upload sessions)
    pub async fn upload_bytes(
        &self,
        url: &str,
        data: &[u8],
        content_range: &str,
        content_length: u64,
    ) -> Result<Response, MogError> {
        let response = self
            .http
            .put(url)
            .header("Content-Range", content_range)
            .header("Content-Length", content_length)
            .body(data.to_vec())
            .timeout(self.upload_timeout)
            .send()
            .await
            .map_err(|e| MogError::Network(format!("Upload failed: {}", e)))?;

        Ok(response)
    }

    fn status_to_error(status: StatusCode, body: &str) -> MogError {
        let msg = extract_error_message(body);
        match status {
            StatusCode::UNAUTHORIZED => MogError::Auth(msg),
            StatusCode::FORBIDDEN => MogError::Authz(msg),
            StatusCode::NOT_FOUND => MogError::NotFound(msg),
            StatusCode::CONFLICT => MogError::Conflict(msg),
            StatusCode::TOO_MANY_REQUESTS => MogError::RateLimited(msg),
            _ => MogError::General(format!("HTTP {}: {}", status.as_u16(), msg)),
        }
    }

    /// GET with automatic pagination, collecting up to max_results (or max_all_results when all=true)
    pub async fn get_collection(
        &self,
        path: &str,
        options: &RequestOptions,
        top: Option<u32>,
        all: bool,
    ) -> Result<Vec<Value>, MogError> {
        let mut results = Vec::new();
        let limit = if all {
            self.max_all_results
        } else {
            self.max_results
        };

        // Add $top to options
        let mut opts = options.clone();
        if let Some(top_val) = top {
            opts.query_params
                .insert("$top".to_string(), top_val.to_string());
        }

        let body = self.request(Method::GET, path, &opts).await?;
        let mut values = pagination::extract_values(&body);
        results.append(&mut values);

        if results.len() as u32 >= limit {
            results.truncate(limit as usize);
            if limit < u32::MAX {
                eprintln!(
                    "Warning: Collection truncated to {} results (limit reached).",
                    limit
                );
            }
            return Ok(results);
        }

        // Follow nextLink
        let mut next_link = pagination::extract_next_link(&body);
        while let Some(link) = next_link {
            if results.len() as u32 >= limit {
                break;
            }

            let normalized = pagination::normalize_next_link(&link, &self.base_url);
            let next_opts = RequestOptions {
                raw_url: Some(normalized),
                headers: options.headers.clone(),
                ..Default::default()
            };
            let next_body = self.request(Method::GET, "", &next_opts).await?;
            let mut next_values = pagination::extract_values(&next_body);
            results.append(&mut next_values);

            if all && results.len() > COLLECTION_SIZE_WARNING_THRESHOLD {
                eprintln!("Warning: Collection exceeds 1,000 results. Use --top to limit. Maximum allowed results: {}", self.max_all_results);
            }
            if all
                && (results.len() as u32) >= self.max_all_results * 9 / 10
                && (results.len() as u32) < self.max_all_results
            {
                eprintln!("Warning: Collection approaching limit of {} results ({}%). Consider using --top to limit further.", self.max_all_results, results.len() as u32 * 100 / self.max_all_results);
            }

            next_link = pagination::extract_next_link(&next_body);
        }

        if results.len() as u32 > limit {
            eprintln!(
                "Warning: Collection truncated to {} results (limit reached).",
                limit
            );
            results.truncate(limit as usize);
        }
        Ok(results)
    }
}

fn extract_error_message(body: &str) -> String {
    if let Ok(json) = serde_json::from_str::<Value>(body) {
        if let Some(error) = json.get("error") {
            let code = error
                .get("code")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let msg = error.get("message").and_then(|v| v.as_str()).unwrap_or("");
            return format!("{}: {}", code, msg);
        }
    }
    if body.is_empty() {
        "No error details".to_string()
    } else {
        body.chars().take(500).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_error_message() {
        let body = r#"{"error":{"code":"ErrorAccessDenied","message":"Access is denied."}}"#;
        assert_eq!(
            extract_error_message(body),
            "ErrorAccessDenied: Access is denied."
        );
    }

    #[test]
    fn test_extract_error_message_empty() {
        assert_eq!(extract_error_message(""), "No error details");
    }

    #[test]
    fn test_build_url() {
        let client = GraphClient::unauthenticated(Cloud::Public, "v1.0").unwrap();
        assert_eq!(
            client.build_url("/me/messages"),
            "https://graph.microsoft.com/v1.0/me/messages"
        );
        assert_eq!(
            client.build_url("me/messages"),
            "https://graph.microsoft.com/v1.0/me/messages"
        );
    }

    #[test]
    fn test_build_url_absolute_passthrough() {
        let client = GraphClient::unauthenticated(Cloud::Public, "v1.0").unwrap();
        let url = "https://graph.microsoft.com/v1.0/me/messages?$skip=25";
        assert_eq!(client.build_url(url), url);
    }

    #[test]
    fn test_build_url_beta() {
        let client = GraphClient::unauthenticated(Cloud::Public, "beta").unwrap();
        assert_eq!(
            client.build_url("me/messages"),
            "https://graph.microsoft.com/beta/me/messages"
        );
    }

    #[test]
    fn test_user_agent_format() {
        let ua = GraphClient::user_agent();
        assert!(ua.starts_with("mog/"));
        assert!(ua.contains("windows") || ua.contains("linux") || ua.contains("macos"));
    }

    #[test]
    fn test_extract_error_message_json() {
        let body = r#"{"error":{"code":"InvalidAuthenticationToken","message":"CompactToken parsing failed."}}"#;
        let msg = extract_error_message(body);
        assert!(msg.contains("InvalidAuthenticationToken"));
        assert!(msg.contains("CompactToken parsing failed"));
    }

    #[test]
    fn test_extract_error_message_plain_text() {
        let msg = extract_error_message("Something went wrong");
        assert_eq!(msg, "Something went wrong");
    }

    #[test]
    fn test_status_to_error_mapping() {
        assert!(matches!(
            GraphClient::status_to_error(StatusCode::UNAUTHORIZED, ""),
            MogError::Auth(_)
        ));
        assert!(matches!(
            GraphClient::status_to_error(StatusCode::FORBIDDEN, ""),
            MogError::Authz(_)
        ));
        assert!(matches!(
            GraphClient::status_to_error(StatusCode::NOT_FOUND, ""),
            MogError::NotFound(_)
        ));
        assert!(matches!(
            GraphClient::status_to_error(StatusCode::TOO_MANY_REQUESTS, ""),
            MogError::RateLimited(_)
        ));
    }
}
