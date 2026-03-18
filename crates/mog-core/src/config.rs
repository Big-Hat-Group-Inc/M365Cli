use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Returns the config directory for mog.
/// Priority: MOG_CONFIG_DIR env > platform default
pub fn config_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("MOG_CONFIG_DIR") {
        return PathBuf::from(dir);
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            return PathBuf::from(local).join("mog");
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Some(home) = dirs::home_dir() {
            return home.join(".config").join("mog");
        }
    }

    // Ultimate fallback
    PathBuf::from(".mog")
}

/// Global configuration (config.json)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub output: OutputConfig,
    #[serde(default)]
    pub graph: GraphConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(rename = "defaultFormat", default = "default_format")]
    pub default_format: String,
    #[serde(default = "default_color")]
    pub color: String,
    #[serde(default)]
    pub pager: Option<String>,
}

fn default_format() -> String {
    "table".into()
}
fn default_color() -> String {
    "auto".into()
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            default_format: default_format(),
            color: default_color(),
            pager: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphConfig {
    #[serde(rename = "defaultApiVersion", default = "default_api_version")]
    pub default_api_version: String,
    #[serde(rename = "maxRetries", default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(rename = "defaultPageSize", default = "default_page_size")]
    pub default_page_size: u32,
    #[serde(rename = "autoPaginate", default = "default_true")]
    pub auto_paginate: bool,
    #[serde(rename = "maxResults", default = "default_max_results")]
    pub max_results: u32,
    #[serde(rename = "maxAllResults", default = "default_max_all_results")]
    pub max_all_results: u32,
    #[serde(rename = "timeoutSeconds", default = "default_timeout_seconds")]
    pub timeout_seconds: u64,
    #[serde(
        rename = "connectTimeoutSeconds",
        default = "default_connect_timeout_seconds"
    )]
    pub connect_timeout_seconds: u64,
    #[serde(
        rename = "uploadTimeoutSeconds",
        default = "default_upload_timeout_seconds"
    )]
    pub upload_timeout_seconds: u64,
    #[serde(rename = "batchSize", default = "default_batch_size")]
    pub batch_size: u32,
}

fn default_api_version() -> String {
    "v1.0".into()
}
fn default_max_retries() -> u32 {
    3
}
fn default_page_size() -> u32 {
    25
}
fn default_true() -> bool {
    true
}
fn default_max_results() -> u32 {
    100
}

fn default_max_all_results() -> u32 {
    10_000
}
fn default_timeout_seconds() -> u64 {
    30
}
fn default_connect_timeout_seconds() -> u64 {
    10
}
fn default_upload_timeout_seconds() -> u64 {
    300
}
fn default_batch_size() -> u32 {
    20
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            default_api_version: default_api_version(),
            max_retries: default_max_retries(),
            default_page_size: default_page_size(),
            auto_paginate: default_true(),
            max_results: default_max_results(),
            max_all_results: default_max_all_results(),
            timeout_seconds: default_timeout_seconds(),
            connect_timeout_seconds: default_connect_timeout_seconds(),
            upload_timeout_seconds: default_upload_timeout_seconds(),
            batch_size: default_batch_size(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(rename = "redactBodies", default = "default_true")]
    pub redact_bodies: bool,
}

fn default_log_level() -> String {
    "warn".into()
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            redact_bodies: true,
        }
    }
}

/// Configuration store — reads/writes config.json
pub struct ConfigStore {
    dir: PathBuf,
}

impl Default for ConfigStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigStore {
    pub fn new() -> Self {
        Self { dir: config_dir() }
    }

    pub fn with_dir(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn config_path(&self) -> PathBuf {
        self.dir.join("config.json")
    }

    pub fn load(&self) -> GlobalConfig {
        let path = self.config_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(config) => config,
                    Err(e) => {
                        tracing::warn!(
                            path = %path.display(),
                            error = %e,
                            "Failed to parse config file, using defaults"
                        );
                        GlobalConfig::default()
                    }
                },
                Err(_) => GlobalConfig::default(),
            }
        } else {
            GlobalConfig::default()
        }
    }

    pub fn save(&self, config: &GlobalConfig) -> Result<(), crate::error::MogError> {
        std::fs::create_dir_all(&self.dir)?;
        let content = serde_json::to_string_pretty(config)?;
        std::fs::write(self.config_path(), content)?;
        Ok(())
    }

    pub fn dir(&self) -> &PathBuf {
        &self.dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GlobalConfig::default();
        assert_eq!(config.output.default_format, "table");
        assert_eq!(config.graph.default_api_version, "v1.0");
        assert_eq!(config.graph.max_retries, 3);
        assert_eq!(config.graph.default_page_size, 25);
        assert_eq!(config.graph.max_results, 100);
    }

    #[test]
    fn test_config_store_roundtrip() {
        let dir = std::env::temp_dir().join("mog-test-config");
        let _ = std::fs::remove_dir_all(&dir);
        let store = ConfigStore::with_dir(dir.clone());
        let config = GlobalConfig::default();
        store.save(&config).unwrap();
        let loaded = store.load();
        assert_eq!(loaded.graph.max_retries, 3);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_config_load_missing_file_returns_defaults() {
        let dir = std::env::temp_dir().join("mog-test-config-missing");
        let _ = std::fs::remove_dir_all(&dir);
        let store = ConfigStore::with_dir(dir.clone());
        let config = store.load();
        assert_eq!(config.output.default_format, "table");
        assert_eq!(config.output.color, "auto");
        assert!(config.output.pager.is_none());
        assert_eq!(config.graph.default_api_version, "v1.0");
        assert!(config.graph.auto_paginate);
        assert_eq!(config.graph.batch_size, 20);
        assert_eq!(config.logging.level, "warn");
        assert!(config.logging.redact_bodies);
    }

    #[test]
    fn test_config_deserialize_partial_json() {
        let json = r#"{"graph": {"maxRetries": 5}}"#;
        let config: GlobalConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.graph.max_retries, 5);
        // Other fields get defaults
        assert_eq!(config.graph.default_api_version, "v1.0");
        assert_eq!(config.output.default_format, "table");
    }
}
