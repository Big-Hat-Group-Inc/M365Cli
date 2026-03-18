use crate::{Cli, TasksCommands};
use mog_core::error::MogError;
use mog_core::output::{OutputFormat, OutputRenderer};
use serde_json::json;

pub async fn run(cli: &Cli, command: &TasksCommands, format: OutputFormat) -> Result<(), MogError> {
    let client =
        super::build_graph_client(cli.profile.as_deref(), &cli.api_version, cli.trace, cli.top)?;

    match command {
        TasksCommands::Lists => {
            let lists = mog_tasks::list_task_lists(&client).await?;

            let output = if format == OutputFormat::Json {
                serde_json::Value::Array(lists)
            } else {
                let summaries: Vec<serde_json::Value> = lists.iter().map(|l| {
                    json!({
                        "id": l.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                        "displayName": l.get("displayName").and_then(|v| v.as_str()).unwrap_or(""),
                    })
                }).collect();
                serde_json::Value::Array(summaries)
            };

            OutputRenderer::render_value(format, &output)?;
            Ok(())
        }

        TasksCommands::List { list_id, status } => {
            let tasks = mog_tasks::list_tasks(&client, list_id, status.as_deref()).await?;

            let output = if format == OutputFormat::Json {
                serde_json::Value::Array(tasks)
            } else {
                let summaries: Vec<serde_json::Value> = tasks.iter().map(|t| {
                    json!({
                        "id": t.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                        "title": t.get("title").and_then(|v| v.as_str()).unwrap_or(""),
                        "status": t.get("status").and_then(|v| v.as_str()).unwrap_or(""),
                        "importance": t.get("importance").and_then(|v| v.as_str()).unwrap_or(""),
                        "due": t.get("dueDateTime")
                            .and_then(|d| d.get("dateTime"))
                            .and_then(|d| d.as_str())
                            .unwrap_or(""),
                    })
                }).collect();
                serde_json::Value::Array(summaries)
            };

            OutputRenderer::render_value(format, &output)?;
            Ok(())
        }

        TasksCommands::Create {
            list_id,
            title,
            body,
            due,
        } => {
            let task =
                mog_tasks::create_task(&client, list_id, title, body.as_deref(), due.as_deref())
                    .await?;
            OutputRenderer::render_value(format, &task)?;
            eprintln!("Task created.");
            Ok(())
        }

        TasksCommands::Update {
            list_id,
            task_id,
            title,
            importance,
        } => {
            let mut fields = serde_json::Map::new();
            if let Some(t) = title {
                fields.insert("title".to_string(), serde_json::Value::String(t.clone()));
            }
            if let Some(i) = importance {
                fields.insert(
                    "importance".to_string(),
                    serde_json::Value::String(i.clone()),
                );
            }
            if fields.is_empty() {
                return Err(MogError::Validation(
                    "No fields to update. Use --title or --importance.".into(),
                ));
            }

            let task = mog_tasks::update_task(
                &client,
                list_id,
                task_id,
                serde_json::Value::Object(fields),
            )
            .await?;
            OutputRenderer::render_value(format, &task)?;
            eprintln!("Task updated.");
            Ok(())
        }

        TasksCommands::Complete { list_id, task_id } => {
            let task = mog_tasks::complete_task(&client, list_id, task_id).await?;
            OutputRenderer::render_value(format, &task)?;
            eprintln!("Task completed.");
            Ok(())
        }

        TasksCommands::Delete { list_id, task_id } => {
            mog_tasks::delete_task(&client, list_id, task_id).await?;
            let result = json!({"status": "deleted", "listId": list_id, "taskId": task_id});
            OutputRenderer::render_value(format, &result)?;
            eprintln!("Task deleted.");
            Ok(())
        }
    }
}
