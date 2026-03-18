use crate::{Cli, GraphCommands};
use mog_core::error::MogError;
use mog_core::output::{OutputFormat, OutputRenderer};
use mog_graph::client::RequestOptions;
use reqwest::Method;
use std::collections::HashMap;
use std::io::Read;

pub async fn run(cli: &Cli, command: &GraphCommands, format: OutputFormat) -> Result<(), MogError> {
    match command {
        GraphCommands::Call {
            verb,
            path,
            select,
            filter,
            header,
            body_file,
            body_stdin,
        } => {
            let client = super::build_graph_client(
                cli.profile.as_deref(),
                &cli.api_version,
                cli.trace,
                cli.top,
            )?;

            // Parse HTTP method
            let method = verb
                .to_uppercase()
                .parse::<Method>()
                .map_err(|_| MogError::Validation(format!("Invalid HTTP method: {}", verb)))?;

            // Build query params
            let mut query_params = HashMap::new();
            if let Some(s) = select {
                query_params.insert("$select".to_string(), s.clone());
            }
            if let Some(f) = filter {
                query_params.insert("$filter".to_string(), f.clone());
            }
            if let Some(top) = cli.top {
                query_params.insert("$top".to_string(), top.to_string());
            }

            // Parse custom headers
            let mut headers = HashMap::new();
            for h in header {
                if let Some((key, value)) = h.split_once(':') {
                    headers.insert(key.trim().to_string(), value.trim().to_string());
                } else {
                    return Err(MogError::Validation(format!(
                        "Invalid header format '{}'. Expected 'Key: Value'",
                        h
                    )));
                }
            }

            // Read body
            let body = if let Some(file) = body_file {
                let content = std::fs::read_to_string(file).map_err(|e| {
                    MogError::General(format!("Failed to read body file '{}': {}", file, e))
                })?;
                let parsed: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
                    MogError::Validation(format!("Invalid JSON in body file: {}", e))
                })?;
                Some(parsed)
            } else if *body_stdin {
                let mut content = String::new();
                std::io::stdin()
                    .read_to_string(&mut content)
                    .map_err(|e| MogError::General(format!("Failed to read from stdin: {}", e)))?;
                if content.trim().is_empty() {
                    None
                } else {
                    let parsed: serde_json::Value =
                        serde_json::from_str(&content).map_err(|e| {
                            MogError::Validation(format!("Invalid JSON from stdin: {}", e))
                        })?;
                    Some(parsed)
                }
            } else {
                None
            };

            let options = RequestOptions {
                query_params,
                headers,
                body,
                ..Default::default()
            };

            // For GET with collections, use pagination-aware method
            if method == Method::GET && cli.all {
                let results = client.get_collection(path, &options, cli.top, true).await?;
                OutputRenderer::render_value(format, &serde_json::Value::Array(results))?;
            } else {
                let result = client.request(method, path, &options).await?;
                OutputRenderer::render_value(format, &result)?;
            }

            Ok(())
        }
    }
}
