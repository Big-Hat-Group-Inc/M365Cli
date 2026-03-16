use crate::{Cli, FilesCommands};
use mog_core::error::MogError;
use mog_core::output::{OutputFormat, OutputRenderer};
use serde_json::json;

pub async fn run(cli: &Cli, command: &FilesCommands, format: OutputFormat) -> Result<(), MogError> {
    let client = super::build_graph_client(
        cli.profile.as_deref(),
        &cli.api_version,
        cli.trace,
        cli.top,
    )?;

    match command {
        FilesCommands::Search { q } => {
            let files = mog_files::search_files(&client, q, cli.top).await?;

            let output = if format == OutputFormat::Json {
                serde_json::Value::Array(files)
            } else {
                let summaries: Vec<serde_json::Value> = files.iter().map(|f| {
                    json!({
                        "id": f.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                        "name": f.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                        "size": f.get("size").and_then(|v| v.as_u64()).unwrap_or(0),
                        "modified": f.get("lastModifiedDateTime").and_then(|v| v.as_str()).unwrap_or(""),
                        "webUrl": f.get("webUrl").and_then(|v| v.as_str()).unwrap_or(""),
                    })
                }).collect();
                serde_json::Value::Array(summaries)
            };

            OutputRenderer::render_value(format, &output)?;
            Ok(())
        }

        FilesCommands::Download { item_id, out } => {
            let path = mog_files::download_file(&client, item_id, out).await?;
            eprintln!("Downloaded to: {}", path);
            let result = json!({"status": "downloaded", "path": path});
            OutputRenderer::render_value(format, &result)?;
            Ok(())
        }

        FilesCommands::Export { item_id, format: export_format, out } => {
            let path = mog_files::export_file(&client, item_id, export_format, out).await?;
            eprintln!("Exported to: {}", path);
            let result = json!({"status": "exported", "path": path, "format": export_format});
            OutputRenderer::render_value(format, &result)?;
            Ok(())
        }

        FilesCommands::Upload { dest, file, resumable, chunk_size } => {
            let result = mog_files::upload_file(
                &client,
                dest,
                file,
                *resumable,
                *chunk_size,
            ).await?;
            eprintln!("Upload complete.");
            OutputRenderer::render_value(format, &result)?;
            Ok(())
        }
    }
}
