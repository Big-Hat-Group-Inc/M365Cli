use mog_core::error::MogError;
use mog_graph::cloud::Cloud;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Auth strategy for a profile
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AuthStrategy {
    #[default]
    DeviceCode,
    Browser,
    ClientCredentials,
    ManagedIdentity,
    Federated,
}

impl std::fmt::Display for AuthStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthStrategy::DeviceCode => write!(f, "device-code"),
            AuthStrategy::Browser => write!(f, "browser"),
            AuthStrategy::ClientCredentials => write!(f, "client-credentials"),
            AuthStrategy::ManagedIdentity => write!(f, "managed-identity"),
            AuthStrategy::Federated => write!(f, "federated"),
        }
    }
}

impl std::str::FromStr for AuthStrategy {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "device-code" | "devicecode" => Ok(AuthStrategy::DeviceCode),
            "browser" => Ok(AuthStrategy::Browser),
            "client-credentials" | "clientcredentials" => Ok(AuthStrategy::ClientCredentials),
            "managed-identity" | "managedidentity" => Ok(AuthStrategy::ManagedIdentity),
            "federated" => Ok(AuthStrategy::Federated),
            _ => Err(format!("Unknown auth strategy: {}", s)),
        }
    }
}

/// A named profile containing auth and connection metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    #[serde(rename = "tenantId")]
    pub tenant_id: String,

    #[serde(rename = "clientId")]
    pub client_id: String,

    #[serde(rename = "authStrategy", default)]
    pub auth_strategy: AuthStrategy,

    #[serde(default)]
    pub cloud: Cloud,

    #[serde(rename = "scopeBundles", default)]
    pub scope_bundles: Vec<String>,

    #[serde(rename = "apiVersion", default = "default_api_version")]
    pub api_version: String,

    #[serde(rename = "certificatePath", skip_serializing_if = "Option::is_none")]
    pub certificate_path: Option<String>,

    #[serde(rename = "certificateFormat", skip_serializing_if = "Option::is_none")]
    pub certificate_format: Option<String>,
}

fn default_api_version() -> String {
    "v1.0".into()
}

/// Profile store (profiles.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileStoreData {
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    pub version: u32,
    #[serde(rename = "defaultProfile")]
    pub default_profile: Option<String>,
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,
}

impl Default for ProfileStoreData {
    fn default() -> Self {
        Self {
            schema: Some("https://mog.dev/schemas/profiles.v1.json".to_string()),
            version: 1,
            default_profile: None,
            profiles: HashMap::new(),
        }
    }
}

/// Manages the profile store on disk
pub struct ProfileStore {
    dir: PathBuf,
}

impl Default for ProfileStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfileStore {
    pub fn new() -> Self {
        Self {
            dir: mog_core::config_dir(),
        }
    }

    pub fn with_dir(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn profiles_path(&self) -> PathBuf {
        self.dir.join("profiles.json")
    }

    pub fn load(&self) -> ProfileStoreData {
        let path = self.profiles_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => ProfileStoreData::default(),
            }
        } else {
            ProfileStoreData::default()
        }
    }

    pub fn save(&self, data: &ProfileStoreData) -> Result<(), MogError> {
        std::fs::create_dir_all(&self.dir)?;
        let content = serde_json::to_string_pretty(data)?;
        std::fs::write(self.profiles_path(), content)?;
        Ok(())
    }

    /// Resolve which profile to use: explicit > env > default
    pub fn resolve_profile_name(&self, explicit: Option<&str>) -> Result<String, MogError> {
        if let Some(name) = explicit {
            return Ok(name.to_string());
        }
        if let Ok(env_profile) = std::env::var("MOG_PROFILE") {
            return Ok(env_profile);
        }
        let data = self.load();
        data.default_profile
            .ok_or_else(|| MogError::Config(
                "No profile specified. Use --profile, MOG_PROFILE env, or set a default profile.".into()
            ))
    }

    /// Get a specific profile
    pub fn get_profile(&self, name: &str) -> Result<Profile, MogError> {
        let data = self.load();
        data.profiles.get(name)
            .cloned()
            .ok_or_else(|| MogError::Config(format!(
                "Profile '{}' not found. Run 'mog auth profile list' to see available profiles.", name
            )))
    }

    /// Create or update a profile
    pub fn upsert_profile(&self, name: &str, profile: Profile) -> Result<(), MogError> {
        let mut data = self.load();
        data.profiles.insert(name.to_string(), profile);
        if data.default_profile.is_none() {
            data.default_profile = Some(name.to_string());
        }
        self.save(&data)
    }

    /// Delete a profile
    pub fn delete_profile(&self, name: &str) -> Result<(), MogError> {
        let mut data = self.load();
        if !data.profiles.contains_key(name) {
            return Err(MogError::NotFound(format!("Profile '{}' not found", name)));
        }
        data.profiles.remove(name);
        if data.default_profile.as_deref() == Some(name) {
            data.default_profile = data.profiles.keys().next().cloned();
        }
        self.save(&data)
    }

    /// Set the default profile
    pub fn set_default(&self, name: &str) -> Result<(), MogError> {
        let mut data = self.load();
        if !data.profiles.contains_key(name) {
            return Err(MogError::NotFound(format!("Profile '{}' not found", name)));
        }
        data.default_profile = Some(name.to_string());
        self.save(&data)
    }

    /// List all profiles
    pub fn list_profiles(&self) -> Vec<(String, Profile, bool)> {
        let data = self.load();
        let default = data.default_profile.as_deref();
        data.profiles
            .into_iter()
            .map(|(name, profile)| {
                let is_default = default == Some(name.as_str());
                (name, profile, is_default)
            })
            .collect()
    }
}

/// Default multi-tenant client ID (placeholder — would be a real app reg in production)
pub const DEFAULT_CLIENT_ID: &str = "00000000-0000-0000-0000-000000000000";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_strategy_display() {
        assert_eq!(AuthStrategy::DeviceCode.to_string(), "device-code");
        assert_eq!(AuthStrategy::Browser.to_string(), "browser");
        assert_eq!(AuthStrategy::ClientCredentials.to_string(), "client-credentials");
    }

    #[test]
    fn test_auth_strategy_from_str() {
        assert_eq!("device-code".parse::<AuthStrategy>().unwrap(), AuthStrategy::DeviceCode);
        assert_eq!("browser".parse::<AuthStrategy>().unwrap(), AuthStrategy::Browser);
        assert!("invalid".parse::<AuthStrategy>().is_err());
    }

    #[test]
    fn test_profile_store_crud() {
        let dir = std::env::temp_dir().join("mog-test-profiles");
        let _ = std::fs::remove_dir_all(&dir);
        let store = ProfileStore::with_dir(dir.clone());

        let profile = Profile {
            tenant_id: "test-tenant".into(),
            client_id: "test-client".into(),
            auth_strategy: AuthStrategy::DeviceCode,
            cloud: Cloud::Public,
            scope_bundles: vec!["mail:read".into()],
            api_version: "v1.0".into(),
            certificate_path: None,
            certificate_format: None,
        };

        store.upsert_profile("test", profile.clone()).unwrap();

        let loaded = store.get_profile("test").unwrap();
        assert_eq!(loaded.tenant_id, "test-tenant");

        let profiles = store.list_profiles();
        assert_eq!(profiles.len(), 1);
        assert!(profiles[0].2); // is default

        store.delete_profile("test").unwrap();
        assert!(store.get_profile("test").is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_resolve_profile_explicit() {
        let dir = std::env::temp_dir().join("mog-test-resolve");
        let _ = std::fs::remove_dir_all(&dir);
        let store = ProfileStore::with_dir(dir.clone());
        let name = store.resolve_profile_name(Some("custom")).unwrap();
        assert_eq!(name, "custom");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_resolve_profile_no_default_errors() {
        let dir = std::env::temp_dir().join("mog-test-resolve-err");
        let _ = std::fs::remove_dir_all(&dir);
        let store = ProfileStore::with_dir(dir.clone());
        // No profiles, no env var, no explicit — should error
        std::env::remove_var("MOG_PROFILE");
        let result = store.resolve_profile_name(None);
        assert!(result.is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_set_default_profile() {
        let dir = std::env::temp_dir().join("mog-test-set-default");
        let _ = std::fs::remove_dir_all(&dir);
        let store = ProfileStore::with_dir(dir.clone());

        let profile = Profile {
            tenant_id: "t".into(),
            client_id: "c".into(),
            auth_strategy: AuthStrategy::DeviceCode,
            cloud: Cloud::Public,
            scope_bundles: vec![],
            api_version: "v1.0".into(),
            certificate_path: None,
            certificate_format: None,
        };

        store.upsert_profile("a", profile.clone()).unwrap();
        store.upsert_profile("b", profile).unwrap();
        store.set_default("b").unwrap();

        let data = store.load();
        assert_eq!(data.default_profile.as_deref(), Some("b"));

        // Can't set default to nonexistent profile
        assert!(store.set_default("nonexistent").is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
