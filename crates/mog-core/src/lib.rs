//! Shared foundation for the `mog` CLI.
//!
//! Provides configuration management ([`ConfigStore`]), a unified error type
//! ([`MogError`]) with CLI exit codes, and multi-format output rendering
//! ([`OutputRenderer`]) for JSON, table, plain-text, and CSV.

pub mod config;
pub mod error;
pub mod output;

pub use config::{config_dir, ConfigStore};
pub use error::{ExitCode, MogError};
pub use output::{OutputFormat, OutputRenderer};
