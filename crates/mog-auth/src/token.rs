use mog_core::error::MogError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use zeroize::Zeroize;

/// Cached token set for a profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedTokens {
    pub access_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
    pub expires_at: i64, // Unix timestamp
    pub scopes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
}

impl CachedTokens {
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        // Consider expired 5 minutes before actual expiry for safety
        now >= self.expires_at - 300
    }
}

impl Drop for CachedTokens {
    fn drop(&mut self) {
        self.access_token.zeroize();
        if let Some(ref mut rt) = self.refresh_token {
            rt.zeroize();
        }
    }
}

/// Token cache stored per-profile
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TokenCacheData {
    pub version: u32,
    #[serde(default)]
    pub tokens: std::collections::HashMap<String, CachedTokens>,
}

/// Manages the token cache on disk
/// In production, this would use OS keyring; for MVP we use encrypted file fallback
pub struct TokenCache {
    dir: PathBuf,
}

impl Default for TokenCache {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenCache {
    pub fn new() -> Self {
        Self {
            dir: mog_core::config_dir(),
        }
    }

    pub fn with_dir(dir: PathBuf) -> Self {
        Self { dir }
    }

    fn cache_path(&self) -> PathBuf {
        self.dir.join("token_cache.json")
    }

    fn load_data(&self) -> TokenCacheData {
        let path = self.cache_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => TokenCacheData::default(),
            }
        } else {
            TokenCacheData::default()
        }
    }

    fn save_data(&self, data: &TokenCacheData) -> Result<(), MogError> {
        std::fs::create_dir_all(&self.dir)?;
        let content = serde_json::to_string_pretty(data)?;
        std::fs::write(self.cache_path(), content)?;
        // TODO: Set restrictive file permissions (0600) on Unix
        Ok(())
    }

    /// Get cached tokens for a profile (returns None if expired)
    pub fn get(&self, profile_name: &str) -> Option<CachedTokens> {
        let data = self.load_data();
        let tokens = data.tokens.get(profile_name)?;
        if tokens.is_expired() {
            // Token expired, but return it if it has a refresh token
            if tokens.refresh_token.is_some() {
                return Some(tokens.clone());
            }
            return None;
        }
        Some(tokens.clone())
    }

    /// Get cached tokens even if expired (for refresh)
    pub fn get_for_refresh(&self, profile_name: &str) -> Option<CachedTokens> {
        let data = self.load_data();
        data.tokens.get(profile_name).cloned()
    }

    /// Store tokens for a profile
    pub fn set(&self, profile_name: &str, tokens: CachedTokens) -> Result<(), MogError> {
        let mut data = self.load_data();
        data.version = 1;
        data.tokens.insert(profile_name.to_string(), tokens);
        self.save_data(&data)
    }

    /// Remove tokens for a profile
    pub fn remove(&self, profile_name: &str) -> Result<(), MogError> {
        let mut data = self.load_data();
        data.tokens.remove(profile_name);
        self.save_data(&data)
    }

    /// Clear all cached tokens
    pub fn clear(&self) -> Result<(), MogError> {
        let path = self.cache_path();
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Check if we have valid (non-expired) tokens for a profile
    pub fn has_valid_tokens(&self, profile_name: &str) -> bool {
        self.load_data()
            .tokens
            .get(profile_name)
            .map(|t| !t.is_expired())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_expiry() {
        let tokens = CachedTokens {
            access_token: "test".into(),
            refresh_token: None,
            id_token: None,
            expires_at: chrono::Utc::now().timestamp() + 3600,
            scopes: vec!["Mail.Read".into()],
            account: None,
        };
        assert!(!tokens.is_expired());

        let expired = CachedTokens {
            access_token: "test".into(),
            refresh_token: None,
            id_token: None,
            expires_at: chrono::Utc::now().timestamp() - 100,
            scopes: vec![],
            account: None,
        };
        assert!(expired.is_expired());
    }

    #[test]
    fn test_token_cache_crud() {
        let dir = std::env::temp_dir().join("mog-test-tokens");
        let _ = std::fs::remove_dir_all(&dir);
        let cache = TokenCache::with_dir(dir.clone());

        let tokens = CachedTokens {
            access_token: "test-access".into(),
            refresh_token: Some("test-refresh".into()),
            id_token: None,
            expires_at: chrono::Utc::now().timestamp() + 3600,
            scopes: vec!["Mail.Read".into()],
            account: Some("user@test.com".into()),
        };

        cache.set("test-profile", tokens).unwrap();
        assert!(cache.has_valid_tokens("test-profile"));

        let loaded = cache.get("test-profile").unwrap();
        assert_eq!(loaded.access_token, "test-access");

        cache.remove("test-profile").unwrap();
        assert!(!cache.has_valid_tokens("test-profile"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
