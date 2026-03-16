use chrono::Datelike;
use mog_core::error::MogError;
use mog_graph::client::{GraphClient, RequestOptions};
use reqwest::Method;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Get today's calendar events via calendarView
pub async fn today(
    client: &GraphClient,
    top: Option<u32>,
) -> Result<Vec<Value>, MogError> {
    let now = chrono::Local::now();
    let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let end = now.date_naive().and_hms_opt(23, 59, 59).unwrap();

    let start_dt = chrono::DateTime::<chrono::Local>::from_naive_utc_and_offset(
        start, *now.offset()
    );
    let end_dt = chrono::DateTime::<chrono::Local>::from_naive_utc_and_offset(
        end, *now.offset()
    );

    calendar_view(
        client,
        &start_dt.to_rfc3339(),
        &end_dt.to_rfc3339(),
        top,
    ).await
}

/// Get this week's calendar events via calendarView
pub async fn week(
    client: &GraphClient,
    top: Option<u32>,
) -> Result<Vec<Value>, MogError> {
    let now = chrono::Local::now();
    let weekday = now.weekday().num_days_from_monday();
    let start_date = now.date_naive() - chrono::Duration::days(weekday as i64);
    let end_date = start_date + chrono::Duration::days(6);

    let start = start_date.and_hms_opt(0, 0, 0).unwrap();
    let end = end_date.and_hms_opt(23, 59, 59).unwrap();

    let start_dt = chrono::DateTime::<chrono::Local>::from_naive_utc_and_offset(
        start, *now.offset()
    );
    let end_dt = chrono::DateTime::<chrono::Local>::from_naive_utc_and_offset(
        end, *now.offset()
    );

    calendar_view(
        client,
        &start_dt.to_rfc3339(),
        &end_dt.to_rfc3339(),
        top,
    ).await
}

/// Get calendar events for a custom range
pub async fn list_events(
    client: &GraphClient,
    start: &str,
    end: &str,
    top: Option<u32>,
) -> Result<Vec<Value>, MogError> {
    // Parse provided dates, add time if missing
    let start_dt = normalize_datetime(start, true);
    let end_dt = normalize_datetime(end, false);

    calendar_view(client, &start_dt, &end_dt, top).await
}

/// Core calendarView query
async fn calendar_view(
    client: &GraphClient,
    start: &str,
    end: &str,
    top: Option<u32>,
) -> Result<Vec<Value>, MogError> {
    let mut params = HashMap::new();
    params.insert("startDateTime".to_string(), start.to_string());
    params.insert("endDateTime".to_string(), end.to_string());
    params.insert(
        "$select".to_string(),
        "id,subject,start,end,location,organizer,attendees,isAllDay,isCancelled,seriesMasterId,type".to_string(),
    );
    params.insert("$orderby".to_string(), "start/dateTime".to_string());

    let options = RequestOptions {
        query_params: params,
        ..Default::default()
    };

    client.get_collection("me/calendarView", &options, top, false).await
}

/// Create a single (non-recurring) event
pub async fn create_event(
    client: &GraphClient,
    subject: &str,
    start: &str,
    end: &str,
    attendees: &[String],
) -> Result<Value, MogError> {
    let start_dt = normalize_datetime(start, true);
    let end_dt = normalize_datetime(end, false);

    let attendee_list: Vec<Value> = attendees.iter().map(|addr| {
        json!({
            "emailAddress": {
                "address": addr
            },
            "type": "required"
        })
    }).collect();

    let event = json!({
        "subject": subject,
        "start": {
            "dateTime": start_dt,
            "timeZone": "UTC"
        },
        "end": {
            "dateTime": end_dt,
            "timeZone": "UTC"
        },
        "attendees": attendee_list
    });

    let options = RequestOptions {
        body: Some(event),
        ..Default::default()
    };

    client.request(Method::POST, "me/events", &options).await
}

/// Update an existing event
pub async fn update_event(
    client: &GraphClient,
    event_id: &str,
    subject: Option<&str>,
    start: Option<&str>,
    end: Option<&str>,
) -> Result<Value, MogError> {
    let mut patch = serde_json::Map::new();

    if let Some(s) = subject {
        patch.insert("subject".to_string(), Value::String(s.to_string()));
    }
    if let Some(s) = start {
        let dt = normalize_datetime(s, true);
        patch.insert("start".to_string(), json!({
            "dateTime": dt,
            "timeZone": "UTC"
        }));
    }
    if let Some(e) = end {
        let dt = normalize_datetime(e, false);
        patch.insert("end".to_string(), json!({
            "dateTime": dt,
            "timeZone": "UTC"
        }));
    }

    if patch.is_empty() {
        return Err(MogError::Validation("No fields to update".into()));
    }

    let options = RequestOptions {
        body: Some(Value::Object(patch)),
        ..Default::default()
    };

    client.request(Method::PATCH, &format!("me/events/{}", event_id), &options).await
}

/// Delete an event
pub async fn delete_event(
    client: &GraphClient,
    event_id: &str,
) -> Result<(), MogError> {
    client.request(Method::DELETE, &format!("me/events/{}", event_id), &RequestOptions::default()).await?;
    Ok(())
}

/// Normalize a datetime string — add T00:00:00 if it's a date-only value
fn normalize_datetime(dt: &str, is_start: bool) -> String {
    if dt.contains('T') || dt.contains(' ') {
        // Already has time component
        dt.to_string()
    } else {
        // Date only — add start/end of day
        if is_start {
            format!("{}T00:00:00", dt)
        } else {
            format!("{}T23:59:59", dt)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_datetime() {
        assert_eq!(normalize_datetime("2025-01-20", true), "2025-01-20T00:00:00");
        assert_eq!(normalize_datetime("2025-01-20", false), "2025-01-20T23:59:59");
        assert_eq!(normalize_datetime("2025-01-20T09:00", true), "2025-01-20T09:00");
    }

    #[test]
    fn test_normalize_datetime_with_space() {
        // Input with space separator should pass through as-is
        assert_eq!(normalize_datetime("2025-01-20 09:00", true), "2025-01-20 09:00");
    }

    #[test]
    fn test_week_range_spans_seven_days() {
        let now = chrono::Local::now();
        let weekday = now.weekday().num_days_from_monday();
        let start_date = now.date_naive() - chrono::Duration::days(weekday as i64);
        let end_date = start_date + chrono::Duration::days(6);
        let diff = (end_date - start_date).num_days();
        assert_eq!(diff, 6);
        // Start should be Monday
        assert_eq!(start_date.weekday(), chrono::Weekday::Mon);
    }

    #[test]
    fn test_today_range_same_day() {
        let now = chrono::Local::now();
        let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let end = now.date_naive().and_hms_opt(23, 59, 59).unwrap();
        assert_eq!(start.date(), end.date());
        assert!(end > start);
    }
}
