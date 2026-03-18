//! Microsoft To Do task operations.
//!
//! List task lists, create, read, update, complete, and delete tasks via
//! the Microsoft Graph To Do API (`/me/todo`).

use mog_core::error::MogError;
use mog_graph::client::{GraphClient, RequestOptions};
use reqwest::Method;
use serde_json::{json, Value};
use std::collections::HashMap;

const DEFAULT_TASK_SELECT: &str =
    "id,title,status,importance,dueDateTime,body,createdDateTime,lastModifiedDateTime";

/// List the user's To Do task lists
pub async fn list_task_lists(client: &GraphClient) -> Result<Vec<Value>, MogError> {
    let options = RequestOptions::default();
    client
        .get_collection("me/todo/lists", &options, None, false)
        .await
}

/// List tasks within a task list, with optional status filter
pub async fn list_tasks(
    client: &GraphClient,
    list_id: &str,
    status_filter: Option<&str>,
) -> Result<Vec<Value>, MogError> {
    let mut params = HashMap::new();
    params.insert("$select".to_string(), DEFAULT_TASK_SELECT.to_string());

    if let Some(status) = status_filter {
        params.insert("$filter".to_string(), format!("status eq '{}'", status));
    }

    let path = format!("me/todo/lists/{}/tasks", list_id);
    let options = RequestOptions {
        query_params: params,
        ..Default::default()
    };

    client.get_collection(&path, &options, None, false).await
}

/// Create a new task in a task list
pub async fn create_task(
    client: &GraphClient,
    list_id: &str,
    title: &str,
    body: Option<&str>,
    due: Option<&str>,
) -> Result<Value, MogError> {
    let mut task = json!({
        "title": title,
    });

    if let Some(b) = body {
        task["body"] = json!({
            "content": b,
            "contentType": "text"
        });
    }

    if let Some(d) = due {
        let due_date = normalize_due_date(d);
        task["dueDateTime"] = json!({
            "dateTime": due_date,
            "timeZone": "UTC"
        });
    }

    let path = format!("me/todo/lists/{}/tasks", list_id);
    let options = RequestOptions {
        body: Some(task),
        ..Default::default()
    };

    client.request(Method::POST, &path, &options).await
}

/// Update an existing task (partial patch)
pub async fn update_task(
    client: &GraphClient,
    list_id: &str,
    task_id: &str,
    fields: Value,
) -> Result<Value, MogError> {
    if fields.as_object().is_none_or(|o| o.is_empty()) {
        return Err(MogError::Validation("No fields to update".into()));
    }

    let path = format!("me/todo/lists/{}/tasks/{}", list_id, task_id);
    let options = RequestOptions {
        body: Some(fields),
        ..Default::default()
    };

    client.request(Method::PATCH, &path, &options).await
}

/// Mark a task as completed
pub async fn complete_task(
    client: &GraphClient,
    list_id: &str,
    task_id: &str,
) -> Result<Value, MogError> {
    let fields = json!({
        "status": "completed"
    });
    update_task(client, list_id, task_id, fields).await
}

/// Delete a task
pub async fn delete_task(
    client: &GraphClient,
    list_id: &str,
    task_id: &str,
) -> Result<(), MogError> {
    let path = format!("me/todo/lists/{}/tasks/{}", list_id, task_id);
    client
        .request(Method::DELETE, &path, &RequestOptions::default())
        .await?;
    Ok(())
}

/// Normalize a due date string — ensure it has a time component
fn normalize_due_date(dt: &str) -> String {
    if dt.contains('T') {
        dt.to_string()
    } else {
        format!("{}T00:00:00", dt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_task_select_fields() {
        assert!(DEFAULT_TASK_SELECT.contains("title"));
        assert!(DEFAULT_TASK_SELECT.contains("status"));
        assert!(DEFAULT_TASK_SELECT.contains("importance"));
        assert!(DEFAULT_TASK_SELECT.contains("dueDateTime"));
    }

    #[test]
    fn test_normalize_due_date_date_only() {
        assert_eq!(normalize_due_date("2025-06-15"), "2025-06-15T00:00:00");
    }

    #[test]
    fn test_normalize_due_date_with_time() {
        assert_eq!(
            normalize_due_date("2025-06-15T09:00:00"),
            "2025-06-15T09:00:00"
        );
    }

    #[test]
    fn test_status_filter_format() {
        let status = "completed";
        let filter = format!("status eq '{}'", status);
        assert_eq!(filter, "status eq 'completed'");
    }
}
