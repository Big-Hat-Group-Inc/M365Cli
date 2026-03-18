use crate::{Cli, DirectoryCommands, DirectoryGroupsCommands, DirectoryUsersCommands};
use mog_core::error::MogError;
use mog_core::output::{OutputFormat, OutputRenderer};
use serde_json::json;

pub async fn run(
    cli: &Cli,
    command: &DirectoryCommands,
    format: OutputFormat,
) -> Result<(), MogError> {
    let client =
        super::build_graph_client(cli.profile.as_deref(), &cli.api_version, cli.trace, cli.top)?;

    match command {
        DirectoryCommands::Users { command: user_cmd } => match user_cmd {
            DirectoryUsersCommands::List { filter } => {
                let users = mog_directory::list_users(&client, cli.top, filter.as_deref()).await?;

                let output = if format == OutputFormat::Json {
                    serde_json::Value::Array(users)
                } else {
                    let summaries: Vec<serde_json::Value> = users.iter().map(|u| {
                            json!({
                                "id": u.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                                "displayName": u.get("displayName").and_then(|v| v.as_str()).unwrap_or(""),
                                "mail": u.get("mail").and_then(|v| v.as_str()).unwrap_or(""),
                                "jobTitle": u.get("jobTitle").and_then(|v| v.as_str()).unwrap_or(""),
                                "department": u.get("department").and_then(|v| v.as_str()).unwrap_or(""),
                            })
                        }).collect();
                    serde_json::Value::Array(summaries)
                };

                OutputRenderer::render_value(format, &output)?;
                Ok(())
            }

            DirectoryUsersCommands::Search { query } => {
                let users = mog_directory::search_users(&client, query, cli.top).await?;

                let output = if format == OutputFormat::Json {
                    serde_json::Value::Array(users)
                } else {
                    let summaries: Vec<serde_json::Value> = users.iter().map(|u| {
                            json!({
                                "id": u.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                                "displayName": u.get("displayName").and_then(|v| v.as_str()).unwrap_or(""),
                                "mail": u.get("mail").and_then(|v| v.as_str()).unwrap_or(""),
                                "jobTitle": u.get("jobTitle").and_then(|v| v.as_str()).unwrap_or(""),
                            })
                        }).collect();
                    serde_json::Value::Array(summaries)
                };

                OutputRenderer::render_value(format, &output)?;
                Ok(())
            }

            DirectoryUsersCommands::Read { id } => {
                let user = mog_directory::read_user(&client, id).await?;
                OutputRenderer::render_value(format, &user)?;
                Ok(())
            }
        },

        DirectoryCommands::Groups { command: group_cmd } => match group_cmd {
            DirectoryGroupsCommands::List { filter } => {
                let groups =
                    mog_directory::list_groups(&client, cli.top, filter.as_deref()).await?;

                let output = if format == OutputFormat::Json {
                    serde_json::Value::Array(groups)
                } else {
                    let summaries: Vec<serde_json::Value> = groups.iter().map(|g| {
                            json!({
                                "id": g.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                                "displayName": g.get("displayName").and_then(|v| v.as_str()).unwrap_or(""),
                                "mail": g.get("mail").and_then(|v| v.as_str()).unwrap_or(""),
                            })
                        }).collect();
                    serde_json::Value::Array(summaries)
                };

                OutputRenderer::render_value(format, &output)?;
                Ok(())
            }

            DirectoryGroupsCommands::Search { query } => {
                let groups = mog_directory::search_groups(&client, query, cli.top).await?;

                let output = if format == OutputFormat::Json {
                    serde_json::Value::Array(groups)
                } else {
                    let summaries: Vec<serde_json::Value> = groups.iter().map(|g| {
                            json!({
                                "id": g.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                                "displayName": g.get("displayName").and_then(|v| v.as_str()).unwrap_or(""),
                                "mail": g.get("mail").and_then(|v| v.as_str()).unwrap_or(""),
                            })
                        }).collect();
                    serde_json::Value::Array(summaries)
                };

                OutputRenderer::render_value(format, &output)?;
                Ok(())
            }

            DirectoryGroupsCommands::Read { id } => {
                let group = mog_directory::read_group(&client, id).await?;
                OutputRenderer::render_value(format, &group)?;
                Ok(())
            }

            DirectoryGroupsCommands::Members { group_id } => {
                let members = mog_directory::list_group_members(&client, group_id).await?;

                let output = if format == OutputFormat::Json {
                    serde_json::Value::Array(members)
                } else {
                    let summaries: Vec<serde_json::Value> = members.iter().map(|m| {
                            json!({
                                "id": m.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                                "displayName": m.get("displayName").and_then(|v| v.as_str()).unwrap_or(""),
                                "mail": m.get("mail").and_then(|v| v.as_str()).unwrap_or(""),
                            })
                        }).collect();
                    serde_json::Value::Array(summaries)
                };

                OutputRenderer::render_value(format, &output)?;
                Ok(())
            }
        },
    }
}
