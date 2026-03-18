//! Authentication flows for mog
//!
//! Implements device code, browser PKCE, client credentials,
//! managed identity, and federated identity flows.

use crate::token::CachedTokens;
use mog_core::error::MogError;
use mog_graph::cloud::Cloud;
use serde::Deserialize;

/// Token response from Entra ID
#[derive(Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub scope: Option<String>,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
}

impl std::fmt::Debug for TokenResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenResponse")
            .field("access_token", &"[REDACTED]")
            .field("token_type", &self.token_type)
            .field("expires_in", &self.expires_in)
            .field("scope", &self.scope)
            .field(
                "refresh_token",
                &self.refresh_token.as_ref().map(|_| "[REDACTED]"),
            )
            .field("id_token", &self.id_token.as_ref().map(|_| "[REDACTED]"))
            .finish()
    }
}

/// Device code response from Entra ID
#[derive(Debug, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
    pub message: String,
}

/// Error response from Entra ID
#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub error_description: Option<String>,
    pub error_codes: Option<Vec<i64>>,
}

/// Execute device code flow
pub async fn device_code_flow(
    client_id: &str,
    tenant_id: &str,
    scopes: &[String],
    cloud: Cloud,
) -> Result<CachedTokens, MogError> {
    let endpoints = cloud.endpoints();
    let device_code_url = format!(
        "{}/{}/oauth2/v2.0/devicecode",
        endpoints.authority, tenant_id
    );
    let token_url = format!("{}/{}/oauth2/v2.0/token", endpoints.authority, tenant_id);

    let scope_string = scopes.join(" ");
    let http = reqwest::Client::new();

    // Step 1: Request device code
    let dc_response = http
        .post(&device_code_url)
        .form(&[("client_id", client_id), ("scope", &scope_string)])
        .send()
        .await
        .map_err(|e| MogError::Network(format!("Device code request failed: {}", e)))?;

    let dc_status = dc_response.status();
    let dc_body = dc_response
        .text()
        .await
        .map_err(|e| MogError::Network(format!("Failed to read device code response: {}", e)))?;

    if !dc_status.is_success() {
        let err: ErrorResponse = serde_json::from_str(&dc_body).unwrap_or(ErrorResponse {
            error: "unknown".into(),
            error_description: Some(dc_body.clone()),
            error_codes: None,
        });
        // Check for CA blocking device code
        if let Some(codes) = &err.error_codes {
            if codes.contains(&50199) || codes.contains(&7000218) {
                return Err(MogError::Auth(
                    "Device code flow is blocked by Conditional Access policy. Try: mog auth login --strategy browser".into()
                ));
            }
        }
        return Err(MogError::Auth(format!(
            "Device code request failed: {} — {}",
            err.error,
            err.error_description.unwrap_or_default()
        )));
    }

    let dc: DeviceCodeResponse = serde_json::from_str(&dc_body)
        .map_err(|e| MogError::Auth(format!("Invalid device code response: {}", e)))?;

    // Display code to user on stderr
    eprintln!("{}", dc.message);
    eprintln!();
    eprintln!("Code: {}", dc.user_code);
    eprintln!("URL:  {}", dc.verification_uri);

    // Step 2: Poll for token
    let interval = std::time::Duration::from_secs(dc.interval.max(5));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(dc.expires_in);

    loop {
        if std::time::Instant::now() > deadline {
            return Err(MogError::Auth(
                "Device code flow timed out (15 minutes)".into(),
            ));
        }

        tokio::time::sleep(interval).await;

        let poll_response = http
            .post(&token_url)
            .form(&[
                ("client_id", client_id),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("device_code", dc.device_code.as_str()),
            ])
            .send()
            .await
            .map_err(|e| MogError::Network(format!("Token poll failed: {}", e)))?;

        let poll_status = poll_response.status();
        let poll_body = poll_response
            .text()
            .await
            .map_err(|e| MogError::Network(format!("Failed to read token response: {}", e)))?;

        if poll_status.is_success() {
            let token: TokenResponse = serde_json::from_str(&poll_body)
                .map_err(|e| MogError::Auth(format!("Invalid token response: {}", e)))?;
            return Ok(token_response_to_cached(token, scopes));
        }

        let err: ErrorResponse = serde_json::from_str(&poll_body).unwrap_or(ErrorResponse {
            error: "unknown".into(),
            error_description: None,
            error_codes: None,
        });

        match err.error.as_str() {
            "authorization_pending" => continue,
            "slow_down" => {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
            "expired_token" => {
                return Err(MogError::Auth(
                    "Device code expired. Please try again.".into(),
                ));
            }
            "authorization_declined" => {
                return Err(MogError::Auth(
                    "Authorization was declined by the user.".into(),
                ));
            }
            _ => {
                return Err(MogError::Auth(format!(
                    "Authentication failed: {} — {}",
                    err.error,
                    err.error_description.unwrap_or_default()
                )));
            }
        }
    }
}

/// Execute client credentials flow (app-only)
pub async fn client_credentials_flow(
    client_id: &str,
    tenant_id: &str,
    client_secret: &str,
    cloud: Cloud,
) -> Result<CachedTokens, MogError> {
    let endpoints = cloud.endpoints();
    let token_url = format!("{}/{}/oauth2/v2.0/token", endpoints.authority, tenant_id);

    let scope = endpoints.resource;
    let http = reqwest::Client::new();

    let response = http
        .post(&token_url)
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("scope", scope),
            ("grant_type", "client_credentials"),
        ])
        .send()
        .await
        .map_err(|e| MogError::Network(format!("Client credentials request failed: {}", e)))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| MogError::Network(format!("Failed to read response: {}", e)))?;

    if !status.is_success() {
        let err: ErrorResponse = serde_json::from_str(&body).unwrap_or(ErrorResponse {
            error: "unknown".into(),
            error_description: Some(body),
            error_codes: None,
        });
        return Err(MogError::Auth(format!(
            "Client credentials failed: {} — {}",
            err.error,
            err.error_description.unwrap_or_default()
        )));
    }

    let token: TokenResponse = serde_json::from_str(&body)
        .map_err(|e| MogError::Auth(format!("Invalid token response: {}", e)))?;

    Ok(CachedTokens {
        access_token: token.access_token,
        refresh_token: None,
        id_token: None,
        expires_at: chrono::Utc::now().timestamp() + token.expires_in as i64,
        scopes: vec![scope.to_string()],
        account: None,
    })
}

/// Execute managed identity flow (IMDS)
pub async fn managed_identity_flow(
    client_id: Option<&str>,
    cloud: Cloud,
) -> Result<CachedTokens, MogError> {
    let endpoints = cloud.endpoints();
    let resource = endpoints.graph;

    let mut url = format!(
        "http://169.254.169.254/metadata/identity/oauth2/token?api-version=2018-02-01&resource={}",
        resource
    );
    if let Some(cid) = client_id {
        url.push_str(&format!("&client_id={}", cid));
    }

    let http = reqwest::Client::new();
    let response = http.get(&url)
        .header("Metadata", "true")
        .send()
        .await
        .map_err(|e| MogError::Auth(format!(
            "Managed identity request failed. Ensure this is running on an Azure resource with managed identity. Error: {}", e
        )))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| MogError::Network(format!("Failed to read IMDS response: {}", e)))?;

    if !status.is_success() {
        return Err(MogError::Auth(format!(
            "Managed identity token acquisition failed ({}): {}",
            status.as_u16(),
            body
        )));
    }

    #[derive(Deserialize)]
    struct ImdsResponse {
        access_token: String,
        expires_in: String,
        #[allow(dead_code)]
        resource: String,
    }

    let imds: ImdsResponse = serde_json::from_str(&body)
        .map_err(|e| MogError::Auth(format!("Invalid IMDS response: {}", e)))?;

    let expires_in: i64 = imds.expires_in.parse().unwrap_or(3600);

    Ok(CachedTokens {
        access_token: imds.access_token,
        refresh_token: None,
        id_token: None,
        expires_at: chrono::Utc::now().timestamp() + expires_in,
        scopes: vec![format!("{}/.default", resource)],
        account: None,
    })
}

/// Execute federated identity flow (token file)
pub async fn federated_identity_flow(
    client_id: &str,
    tenant_id: &str,
    token_file: &str,
    cloud: Cloud,
) -> Result<CachedTokens, MogError> {
    let endpoints = cloud.endpoints();
    let token_url = format!("{}/{}/oauth2/v2.0/token", endpoints.authority, tenant_id);

    let assertion = std::fs::read_to_string(token_file).map_err(|e| {
        MogError::Auth(format!(
            "Failed to read federated token file '{}': {}",
            token_file, e
        ))
    })?;
    let assertion = assertion.trim().to_string();

    let scope = endpoints.resource;
    let http = reqwest::Client::new();

    let response = http
        .post(&token_url)
        .form(&[
            ("client_id", client_id),
            ("scope", scope),
            ("grant_type", "client_credentials"),
            (
                "client_assertion_type",
                "urn:ietf:params:oauth:client-assertion-type:jwt-bearer",
            ),
            ("client_assertion", assertion.as_str()),
        ])
        .send()
        .await
        .map_err(|e| MogError::Network(format!("Federated token request failed: {}", e)))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| MogError::Network(format!("Failed to read response: {}", e)))?;

    if !status.is_success() {
        let err: ErrorResponse = serde_json::from_str(&body).unwrap_or(ErrorResponse {
            error: "unknown".into(),
            error_description: Some(body),
            error_codes: None,
        });
        return Err(MogError::Auth(format!(
            "Federated identity failed: {} — {}",
            err.error,
            err.error_description.unwrap_or_default()
        )));
    }

    let token: TokenResponse = serde_json::from_str(&body)
        .map_err(|e| MogError::Auth(format!("Invalid token response: {}", e)))?;

    Ok(CachedTokens {
        access_token: token.access_token,
        refresh_token: None,
        id_token: None,
        expires_at: chrono::Utc::now().timestamp() + token.expires_in as i64,
        scopes: vec![scope.to_string()],
        account: None,
    })
}

/// Refresh an existing token using a refresh token
pub async fn refresh_token_flow(
    client_id: &str,
    tenant_id: &str,
    refresh_token: &str,
    scopes: &[String],
    cloud: Cloud,
) -> Result<CachedTokens, MogError> {
    let endpoints = cloud.endpoints();
    let token_url = format!("{}/{}/oauth2/v2.0/token", endpoints.authority, tenant_id);

    let scope_string = scopes.join(" ");
    let http = reqwest::Client::new();

    let response = http
        .post(&token_url)
        .form(&[
            ("client_id", client_id),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("scope", scope_string.as_str()),
        ])
        .send()
        .await
        .map_err(|e| MogError::Network(format!("Token refresh failed: {}", e)))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| MogError::Network(format!("Failed to read response: {}", e)))?;

    if !status.is_success() {
        let err: ErrorResponse = serde_json::from_str(&body).unwrap_or(ErrorResponse {
            error: "unknown".into(),
            error_description: Some(body),
            error_codes: None,
        });
        if err.error == "invalid_grant" {
            return Err(MogError::Auth(
                "Refresh token expired or revoked. Please re-authenticate: mog auth login".into(),
            ));
        }
        return Err(MogError::Auth(format!(
            "Token refresh failed: {} — {}",
            err.error,
            err.error_description.unwrap_or_default()
        )));
    }

    let token: TokenResponse = serde_json::from_str(&body)
        .map_err(|e| MogError::Auth(format!("Invalid token response: {}", e)))?;

    Ok(token_response_to_cached(token, scopes))
}

fn token_response_to_cached(token: TokenResponse, scopes: &[String]) -> CachedTokens {
    CachedTokens {
        access_token: token.access_token,
        refresh_token: token.refresh_token,
        id_token: token.id_token,
        expires_at: chrono::Utc::now().timestamp() + token.expires_in as i64,
        scopes: token
            .scope
            .map(|s| s.split(' ').map(|x| x.to_string()).collect())
            .unwrap_or_else(|| scopes.to_vec()),
        account: None,
    }
}
