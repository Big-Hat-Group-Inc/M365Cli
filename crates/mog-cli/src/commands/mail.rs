use crate::{AttachmentCommands, Cli, MailCommands};
use mog_core::error::MogError;
use mog_core::output::{OutputFormat, OutputRenderer};
use serde_json::json;

pub async fn run(cli: &Cli, command: &MailCommands, format: OutputFormat) -> Result<(), MogError> {
    let client =
        super::build_graph_client(cli.profile.as_deref(), &cli.api_version, cli.trace, cli.top)?;

    match command {
        MailCommands::List {
            unread,
            since,
            include_body,
        } => {
            let messages = mog_mail::list_messages(
                &client,
                *unread,
                since.as_deref(),
                cli.top,
                *include_body,
                cli.all,
            )
            .await?;

            // Transform to summary format for table/plain output
            let output = if format == OutputFormat::Json {
                serde_json::Value::Array(messages)
            } else {
                let summaries: Vec<serde_json::Value> = messages.iter().map(|m| {
                    json!({
                        "id": m.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                        "from": extract_from(m),
                        "subject": m.get("subject").and_then(|v| v.as_str()).unwrap_or("(no subject)"),
                        "date": m.get("receivedDateTime").and_then(|v| v.as_str()).unwrap_or(""),
                        "preview": truncate(
                            m.get("bodyPreview").and_then(|v| v.as_str()).unwrap_or(""),
                            200
                        ),
                        "read": m.get("isRead").and_then(|v| v.as_bool()).unwrap_or(true),
                    })
                }).collect();
                serde_json::Value::Array(summaries)
            };

            OutputRenderer::render_value(format, &output)?;
            Ok(())
        }

        MailCommands::Search { kql } => {
            let messages = mog_mail::search_messages(&client, kql, cli.top).await?;
            let output = serde_json::Value::Array(messages);
            OutputRenderer::render_value(format, &output)?;
            Ok(())
        }

        MailCommands::Read { id, body_type } => {
            let message = mog_mail::read_message(&client, id, body_type.as_deref()).await?;
            OutputRenderer::render_value(format, &message)?;
            Ok(())
        }

        MailCommands::Send {
            to,
            subject,
            body_file,
            attach,
        } => {
            let body_content = if let Some(file) = body_file {
                std::fs::read_to_string(file).map_err(|e| {
                    MogError::General(format!("Failed to read body file '{}': {}", file, e))
                })?
            } else {
                String::new()
            };

            let result =
                mog_mail::send_message(&client, to, subject, &body_content, attach).await?;
            OutputRenderer::render_value(format, &result)?;
            eprintln!("Message sent.");
            Ok(())
        }

        MailCommands::Attachments { command } => match command {
            AttachmentCommands::List { id } => {
                let attachments = mog_mail::list_attachments(&client, id).await?;
                OutputRenderer::render_value(format, &serde_json::Value::Array(attachments))?;
                Ok(())
            }
            AttachmentCommands::Download {
                id,
                out_dir,
                attachment_id,
                out,
            } => {
                let dir = out_dir.as_deref().or(out.as_deref()).unwrap_or(".");
                let downloaded =
                    mog_mail::download_attachments(&client, id, dir, attachment_id.as_deref())
                        .await?;

                for path in &downloaded {
                    eprintln!("Downloaded: {}", path);
                }

                let result = json!({"downloaded": downloaded});
                OutputRenderer::render_value(format, &result)?;
                Ok(())
            }
        },
    }
}

fn extract_from(msg: &serde_json::Value) -> String {
    msg.get("from")
        .and_then(|f| f.get("emailAddress"))
        .and_then(|ea| ea.get("address"))
        .and_then(|a| a.as_str())
        .unwrap_or("unknown")
        .to_string()
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}
