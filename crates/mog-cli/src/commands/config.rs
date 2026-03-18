use crate::ConfigCommands;
use mog_core::error::MogError;
use mog_core::output::{OutputFormat, OutputRenderer};
use mog_core::ConfigStore;

pub async fn run(command: &ConfigCommands, format: OutputFormat) -> Result<(), MogError> {
    match command {
        ConfigCommands::Show => {
            let store = ConfigStore::new();
            let config = store.load();
            let value = serde_json::to_value(&config)?;
            OutputRenderer::render_value(format, &value)?;
            Ok(())
        }
        ConfigCommands::Set { key, value } => {
            let store = ConfigStore::new();
            let mut config = store.load();

            match key.as_str() {
                "output.defaultFormat" => config.output.default_format = value.clone(),
                "output.color" => config.output.color = value.clone(),
                "graph.defaultApiVersion" => config.graph.default_api_version = value.clone(),
                "graph.maxRetries" => {
                    config.graph.max_retries = value
                        .parse()
                        .map_err(|_| MogError::Validation(format!("Invalid number: {}", value)))?;
                }
                "graph.defaultPageSize" => {
                    config.graph.default_page_size = value
                        .parse()
                        .map_err(|_| MogError::Validation(format!("Invalid number: {}", value)))?;
                }
                "graph.maxResults" => {
                    config.graph.max_results = value
                        .parse()
                        .map_err(|_| MogError::Validation(format!("Invalid number: {}", value)))?;
                }
                "logging.level" => config.logging.level = value.clone(),
                "logging.redactBodies" => {
                    config.logging.redact_bodies = value
                        .parse()
                        .map_err(|_| MogError::Validation(format!("Invalid boolean: {}", value)))?;
                }
                _ => return Err(MogError::Validation(format!("Unknown config key: {}", key))),
            }

            store.save(&config)?;
            eprintln!("Set {} = {}", key, value);
            Ok(())
        }
    }
}
