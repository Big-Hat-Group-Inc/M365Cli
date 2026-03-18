use fs2::FileExt;
use mog_core::error::MogError;
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, Write};
use std::path::PathBuf;
use zeroize::Zeroize;

/// Buffer in seconds before actual token expiry to consider a token expired.
/// Prevents using a token that is about to expire during an in-flight request.
const TOKEN_EXPIRY_BUFFER_SECS: i64 = 300;

/// Cached token set for a profile
#[derive(Clone, Serialize, Deserialize)]
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

impl std::fmt::Debug for CachedTokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CachedTokens")
            .field("access_token", &"[REDACTED]")
            .field(
                "refresh_token",
                &self.refresh_token.as_ref().map(|_| "[REDACTED]"),
            )
            .field("id_token", &self.id_token.as_ref().map(|_| "[REDACTED]"))
            .field("expires_at", &self.expires_at)
            .field("scopes", &self.scopes)
            .field("account", &self.account)
            .finish()
    }
}

impl CachedTokens {
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        now >= self.expires_at - TOKEN_EXPIRY_BUFFER_SECS
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
#[derive(Serialize, Deserialize, Default)]
pub struct TokenCacheData {
    pub version: u32,
    #[serde(default)]
    pub tokens: std::collections::HashMap<String, CachedTokens>,
}

impl std::fmt::Debug for TokenCacheData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenCacheData")
            .field("version", &self.version)
            .field("cached_profiles", &self.tokens.len())
            .finish()
    }
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

    fn with_shared_lock<F, R>(&self, f: F) -> Result<R, MogError>
    where
        F: FnOnce(&TokenCacheData) -> Result<R, MogError>,
    {
        std::fs::create_dir_all(&self.dir)?;
        let path = self.cache_path();
        let mut file = match std::fs::OpenOptions::new().read(true).open(&path) {
            Ok(file) => file,
            Err(_) => {
                // File doesn't exist, create it with restrictive permissions
                let file = std::fs::File::create(&path)?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = file.metadata()?.permissions();
                    perms.set_mode(0o600);
                    if let Err(e) = file.set_permissions(perms) {
                        tracing::warn!(
                            "Failed to set file permissions on {}: {}",
                            path.display(),
                            e
                        );
                    }
                }
                file
            }
        };
        file.lock_shared()?;

        // If file is empty (newly created), ensure permissions are set
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if file.metadata()?.len() == 0 {
                let mut perms = file.metadata()?.permissions();
                perms.set_mode(0o600);
                if let Err(e) = file.set_permissions(perms) {
                    tracing::warn!(
                        "Failed to set file permissions on {}: {}",
                        path.display(),
                        e
                    );
                }
            }
        }

        let data = if file.metadata()?.len() > 0 {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            match serde_json::from_str(&content) {
                Ok(parsed) => parsed,
                Err(e) => {
                    tracing::warn!(
                        path = %path.display(),
                        error = %e,
                        "Failed to parse token cache, using defaults"
                    );
                    TokenCacheData::default()
                }
            }
        } else {
            TokenCacheData::default()
        };

        let result = f(&data);
        file.unlock()?;
        result
    }

    fn with_exclusive_lock<F, R>(&self, f: F) -> Result<R, MogError>
    where
        F: FnOnce(&mut TokenCacheData) -> Result<R, MogError>,
    {
        std::fs::create_dir_all(&self.dir)?;
        let path = self.cache_path();
        // We intentionally open without truncate: we read first, then
        // seek-to-start + set_len(0) + write-back under exclusive lock.
        #[allow(clippy::suspicious_open_options)]
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;
        file.lock_exclusive()?;

        // Set permissions on newly created empty file
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if file.metadata()?.len() == 0 {
                let mut perms = file.metadata()?.permissions();
                perms.set_mode(0o600);
                if let Err(e) = file.set_permissions(perms) {
                    tracing::warn!(
                        "Failed to set file permissions on {}: {}",
                        path.display(),
                        e
                    );
                }
            }
        }

        let mut data = if file.metadata()?.len() > 0 {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            match serde_json::from_str(&content) {
                Ok(parsed) => parsed,
                Err(e) => {
                    tracing::warn!(
                        path = %path.display(),
                        error = %e,
                        "Failed to parse token cache, using defaults"
                    );
                    TokenCacheData::default()
                }
            }
        } else {
            TokenCacheData::default()
        };

        let result = f(&mut data);

        if result.is_ok() {
            // Write back to the same file handle
            file.seek(std::io::SeekFrom::Start(0))?;
            file.set_len(0)?;
            let content = serde_json::to_string_pretty(&data)?;
            file.write_all(content.as_bytes())?;

            // Ensure restrictive file permissions (0600) on Unix after write
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(metadata) = file.metadata() {
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o600);
                    if let Err(e) = file.set_permissions(perms) {
                        tracing::warn!(
                            "Failed to set file permissions on {}: {}",
                            path.display(),
                            e
                        );
                    }
                }
            }
        }

        file.unlock()?;
        result
    }

    /// Get cached tokens for a profile (returns None if expired)
    pub fn get(&self, profile_name: &str) -> Option<CachedTokens> {
        let result = self.with_shared_lock(|data| {
            let tokens = match data.tokens.get(profile_name) {
                Some(t) => t,
                None => return Ok(None),
            };
            if tokens.is_expired() {
                // Token expired, but return it if it has a refresh token
                if tokens.refresh_token.is_some() {
                    return Ok(Some(tokens.clone()));
                }
                return Ok(None);
            }
            Ok(Some(tokens.clone()))
        });
        result.unwrap_or_default()
    }

    /// Get cached tokens even if expired (for refresh)
    pub fn get_for_refresh(&self, profile_name: &str) -> Option<CachedTokens> {
        let result = self.with_shared_lock(|data| Ok(data.tokens.get(profile_name).cloned()));
        result.unwrap_or_default()
    }

    /// Store tokens for a profile
    pub fn set(&self, profile_name: &str, tokens: CachedTokens) -> Result<(), MogError> {
        self.with_exclusive_lock(|data| {
            data.version = 1;
            data.tokens.insert(profile_name.to_string(), tokens);
            Ok(())
        })
    }

    /// Remove tokens for a profile
    pub fn remove(&self, profile_name: &str) -> Result<(), MogError> {
        self.with_exclusive_lock(|data| {
            data.tokens.remove(profile_name);
            Ok(())
        })
    }

    /// Clear all cached tokens
    pub fn clear(&self) -> Result<(), MogError> {
        let path = self.cache_path();
        if path.exists() {
            // Lock the file before removing to prevent concurrent reads
            let file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&path)?;
            file.lock_exclusive()?;
            std::fs::remove_file(&path)?;
            // Lock is released when file is closed
        }
        Ok(())
    }

    /// Check if we have valid (non-expired) tokens for a profile
    pub fn has_valid_tokens(&self, profile_name: &str) -> bool {
        let result = self.with_shared_lock(|data| {
            Ok(data
                .tokens
                .get(profile_name)
                .map(|t| !t.is_expired())
                .unwrap_or(false))
        });
        result.unwrap_or_default()
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
