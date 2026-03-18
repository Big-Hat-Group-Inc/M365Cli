use crate::{Cli, PeopleCommands};
use mog_core::error::MogError;
use mog_core::output::{OutputFormat, OutputRenderer};
use serde_json::json;

pub async fn run(
    cli: &Cli,
    command: &PeopleCommands,
    format: OutputFormat,
) -> Result<(), MogError> {
    let client =
        super::build_graph_client(cli.profile.as_deref(), &cli.api_version, cli.trace, cli.top)?;

    match command {
        PeopleCommands::Relevant => {
            let people = mog_people::relevant_people(&client, cli.top).await?;
            let output = format_people(people, format);
            OutputRenderer::render_value(format, &output)?;
            Ok(())
        }

        PeopleCommands::Search { query } => {
            let people = mog_people::search_people(&client, query, cli.top).await?;
            let output = format_people(people, format);
            OutputRenderer::render_value(format, &output)?;
            Ok(())
        }
    }
}

fn format_people(people: Vec<serde_json::Value>, format: OutputFormat) -> serde_json::Value {
    if format == OutputFormat::Json {
        return serde_json::Value::Array(people);
    }

    let summaries: Vec<serde_json::Value> = people
        .iter()
        .map(|p| {
            json!({
                "id": p.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                "displayName": p.get("displayName").and_then(|v| v.as_str()).unwrap_or(""),
                "email": mog_people::extract_primary_email(p),
                "jobTitle": p.get("jobTitle").and_then(|v| v.as_str()).unwrap_or(""),
                "department": p.get("department").and_then(|v| v.as_str()).unwrap_or(""),
            })
        })
        .collect();

    serde_json::Value::Array(summaries)
}
