use crate::{Cli, ContactsCommands};
use mog_core::error::MogError;
use mog_core::output::{OutputFormat, OutputRenderer};
use serde_json::json;

pub async fn run(
    cli: &Cli,
    command: &ContactsCommands,
    format: OutputFormat,
) -> Result<(), MogError> {
    let client =
        super::build_graph_client(cli.profile.as_deref(), &cli.api_version, cli.trace, cli.top)?;

    match command {
        ContactsCommands::List { filter } => {
            let contacts = mog_contacts::list_contacts(&client, cli.top, filter.as_deref()).await?;

            let output = if format == OutputFormat::Json {
                serde_json::Value::Array(contacts)
            } else {
                let summaries: Vec<serde_json::Value> = contacts.iter().map(|c| {
                    json!({
                        "id": c.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                        "displayName": c.get("displayName").and_then(|v| v.as_str()).unwrap_or(""),
                        "email": mog_contacts::extract_primary_email(c),
                        "phone": c.get("mobilePhone").and_then(|v| v.as_str()).unwrap_or(""),
                        "company": c.get("companyName").and_then(|v| v.as_str()).unwrap_or(""),
                    })
                }).collect();
                serde_json::Value::Array(summaries)
            };

            OutputRenderer::render_value(format, &output)?;
            Ok(())
        }

        ContactsCommands::Search { query } => {
            let contacts = mog_contacts::search_contacts(&client, query, cli.top).await?;

            let output = if format == OutputFormat::Json {
                serde_json::Value::Array(contacts)
            } else {
                let summaries: Vec<serde_json::Value> = contacts.iter().map(|c| {
                    json!({
                        "id": c.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                        "displayName": c.get("displayName").and_then(|v| v.as_str()).unwrap_or(""),
                        "email": mog_contacts::extract_primary_email(c),
                        "phone": c.get("mobilePhone").and_then(|v| v.as_str()).unwrap_or(""),
                    })
                }).collect();
                serde_json::Value::Array(summaries)
            };

            OutputRenderer::render_value(format, &output)?;
            Ok(())
        }

        ContactsCommands::Read { id } => {
            let contact = mog_contacts::read_contact(&client, id).await?;
            OutputRenderer::render_value(format, &contact)?;
            Ok(())
        }
    }
}
