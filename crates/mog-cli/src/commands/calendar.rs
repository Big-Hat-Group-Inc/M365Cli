use crate::{CalendarCommands, Cli};
use mog_core::error::MogError;
use mog_core::output::{OutputFormat, OutputRenderer};
use serde_json::json;

pub async fn run(cli: &Cli, command: &CalendarCommands, format: OutputFormat) -> Result<(), MogError> {
    let client = super::build_graph_client(
        cli.profile.as_deref(),
        &cli.api_version,
        cli.trace,
        cli.top,
    )?;

    match command {
        CalendarCommands::Today => {
            let events = mog_calendar::today(&client, cli.top).await?;
            let output = format_events(events, format);
            OutputRenderer::render_value(format, &output)?;
            Ok(())
        }

        CalendarCommands::Week => {
            let events = mog_calendar::week(&client, cli.top).await?;
            let output = format_events(events, format);
            OutputRenderer::render_value(format, &output)?;
            Ok(())
        }

        CalendarCommands::List { start, end } => {
            let events = mog_calendar::list_events(&client, start, end, cli.top).await?;
            let output = format_events(events, format);
            OutputRenderer::render_value(format, &output)?;
            Ok(())
        }

        CalendarCommands::Create { subject, start, end, attendees } => {
            let attendee_list = attendees.as_deref().unwrap_or(&[]);
            let event = mog_calendar::create_event(
                &client,
                subject,
                start,
                end,
                attendee_list,
            ).await?;
            OutputRenderer::render_value(format, &event)?;
            eprintln!("Event created.");
            Ok(())
        }

        CalendarCommands::Update { id, subject, start, end } => {
            let event = mog_calendar::update_event(
                &client,
                id,
                subject.as_deref(),
                start.as_deref(),
                end.as_deref(),
            ).await?;
            OutputRenderer::render_value(format, &event)?;
            eprintln!("Event updated.");
            Ok(())
        }

        CalendarCommands::Delete { id } => {
            mog_calendar::delete_event(&client, id).await?;
            let result = json!({"status": "deleted", "id": id});
            OutputRenderer::render_value(format, &result)?;
            eprintln!("Event deleted.");
            Ok(())
        }
    }
}

fn format_events(events: Vec<serde_json::Value>, format: OutputFormat) -> serde_json::Value {
    if format == OutputFormat::Json {
        return serde_json::Value::Array(events);
    }

    let summaries: Vec<serde_json::Value> = events.iter().map(|e| {
        let start = e.get("start")
            .and_then(|s| s.get("dateTime"))
            .and_then(|d| d.as_str())
            .unwrap_or("");
        let end = e.get("end")
            .and_then(|s| s.get("dateTime"))
            .and_then(|d| d.as_str())
            .unwrap_or("");
        let location = e.get("location")
            .and_then(|l| l.get("displayName"))
            .and_then(|d| d.as_str())
            .unwrap_or("");

        json!({
            "id": e.get("id").and_then(|v| v.as_str()).unwrap_or(""),
            "subject": e.get("subject").and_then(|v| v.as_str()).unwrap_or("(no subject)"),
            "start": start,
            "end": end,
            "location": location,
            "allDay": e.get("isAllDay").and_then(|v| v.as_bool()).unwrap_or(false),
        })
    }).collect();

    serde_json::Value::Array(summaries)
}
