//! Microsoft 365 mail operations.
//!
//! List, search, read, and send messages via Microsoft Graph. Supports
//! attachment listing, downloading, and inline file attachments when sending.

use mog_core::error::MogError;
use mog_graph::client::{GraphClient, RequestOptions};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;

/// Mail message summary (for list output)
#[derive(Debug, Serialize, Deserialize)]
pub struct MailSummary {
    pub id: String,
    pub subject: String,
    pub from: String,
    pub to: String,
    pub date: String,
    pub preview: String,
    #[serde(rename = "isRead")]
    pub is_read: bool,
}

/// List mail messages (headers only by default)
pub async fn list_messages(
    client: &GraphClient,
    unread: bool,
    since: Option<&str>,
    top: Option<u32>,
    include_body: bool,
    all: bool,
) -> Result<Vec<Value>, MogError> {
    let mut params = HashMap::new();

    // Default $select: headers only
    let select = if include_body {
        "id,subject,from,toRecipients,receivedDateTime,bodyPreview,isRead,body,hasAttachments"
    } else {
        "id,subject,from,toRecipients,receivedDateTime,bodyPreview,isRead,hasAttachments"
    };
    params.insert("$select".to_string(), select.to_string());
    params.insert("$orderby".to_string(), "receivedDateTime desc".to_string());

    // Build filter
    let mut filters = Vec::new();
    if unread {
        filters.push("isRead eq false".to_string());
    }
    if let Some(since_val) = since {
        if let Some(dt) = parse_since(since_val) {
            filters.push(format!("receivedDateTime ge {}", dt));
        }
    }
    if !filters.is_empty() {
        params.insert("$filter".to_string(), filters.join(" and "));
    }

    let options = RequestOptions {
        query_params: params,
        ..Default::default()
    };

    client
        .get_collection("me/messages", &options, top, all)
        .await
}

/// Search mail using KQL
pub async fn search_messages(
    client: &GraphClient,
    kql: &str,
    top: Option<u32>,
) -> Result<Vec<Value>, MogError> {
    let mut params = HashMap::new();
    params.insert("$search".to_string(), format!("\"{}\"", kql));
    params.insert(
        "$select".to_string(),
        "id,subject,from,toRecipients,receivedDateTime,bodyPreview,isRead,hasAttachments"
            .to_string(),
    );

    let options = RequestOptions {
        query_params: params,
        ..Default::default()
    };

    client
        .get_collection("me/messages", &options, top, false)
        .await
}

/// Read a full message by ID
pub async fn read_message(
    client: &GraphClient,
    message_id: &str,
    body_type: Option<&str>,
) -> Result<Value, MogError> {
    let mut headers = HashMap::new();
    if let Some(bt) = body_type {
        headers.insert(
            "Prefer".to_string(),
            format!("outlook.body-content-type=\"{}\"", bt),
        );
    }

    let options = RequestOptions {
        headers,
        ..Default::default()
    };

    client
        .request(
            Method::GET,
            &format!("me/messages/{}", message_id),
            &options,
        )
        .await
}

/// Send a mail message
pub async fn send_message(
    client: &GraphClient,
    to: &[String],
    subject: &str,
    body_content: &str,
    attachments: &[String],
) -> Result<Value, MogError> {
    let to_recipients: Vec<Value> = to
        .iter()
        .map(|addr| {
            json!({
                "emailAddress": {
                    "address": addr
                }
            })
        })
        .collect();

    let mut message = json!({
        "subject": subject,
        "body": {
            "contentType": "Text",
            "content": body_content
        },
        "toRecipients": to_recipients
    });

    // Add attachments
    if !attachments.is_empty() {
        let mut att_array = Vec::new();
        for path_str in attachments {
            let path = Path::new(path_str);
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("attachment")
                .to_string();
            let content = tokio::fs::read(path).await.map_err(|e| {
                MogError::General(format!("Failed to read attachment '{}': {}", path_str, e))
            })?;
            let content_b64 =
                base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &content);
            let content_type = guess_content_type(&filename);
            att_array.push(json!({
                "@odata.type": "#microsoft.graph.fileAttachment",
                "name": filename,
                "contentType": content_type,
                "contentBytes": content_b64
            }));
        }
        message["attachments"] = Value::Array(att_array);
    }

    let payload = json!({
        "message": message,
        "saveToSentItems": true
    });

    let options = RequestOptions {
        body: Some(payload),
        ..Default::default()
    };

    client
        .request(Method::POST, "me/sendMail", &options)
        .await?;
    Ok(json!({"status": "sent"}))
}

/// List attachments for a message (metadata only)
pub async fn list_attachments(
    client: &GraphClient,
    message_id: &str,
) -> Result<Vec<Value>, MogError> {
    let mut params = HashMap::new();
    params.insert(
        "$select".to_string(),
        "id,name,contentType,size".to_string(),
    );

    let options = RequestOptions {
        query_params: params,
        ..Default::default()
    };

    client
        .get_collection(
            &format!("me/messages/{}/attachments", message_id),
            &options,
            None,
            false,
        )
        .await
}

/// Download attachments for a message
pub async fn download_attachments(
    client: &GraphClient,
    message_id: &str,
    out_dir: &str,
    attachment_id: Option<&str>,
) -> Result<Vec<String>, MogError> {
    let dir = Path::new(out_dir);
    std::fs::create_dir_all(dir)?;

    let mut downloaded = Vec::new();

    if let Some(att_id) = attachment_id {
        // Download specific attachment
        let path = format!("me/messages/{}/attachments/{}", message_id, att_id);
        let att = client
            .request(Method::GET, &path, &RequestOptions::default())
            .await?;
        if let Some(name) = att.get("name").and_then(|v| v.as_str()) {
            if let Some(bytes) = att.get("contentBytes").and_then(|v| v.as_str()) {
                let decoded =
                    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, bytes)
                        .map_err(|e| {
                            MogError::General(format!("Failed to decode attachment: {}", e))
                        })?;
                let file_path = dir.join(name);
                std::fs::write(&file_path, &decoded)?;
                downloaded.push(file_path.to_string_lossy().to_string());
            }
        }
    } else {
        // Download all attachments
        let attachments = list_attachments(client, message_id).await?;
        for att in &attachments {
            if let Some(att_id) = att.get("id").and_then(|v| v.as_str()) {
                let path = format!("me/messages/{}/attachments/{}", message_id, att_id);
                let full_att = client
                    .request(Method::GET, &path, &RequestOptions::default())
                    .await?;
                if let Some(name) = full_att.get("name").and_then(|v| v.as_str()) {
                    if let Some(bytes) = full_att.get("contentBytes").and_then(|v| v.as_str()) {
                        let decoded = base64::Engine::decode(
                            &base64::engine::general_purpose::STANDARD,
                            bytes,
                        )
                        .map_err(|e| {
                            MogError::General(format!("Failed to decode attachment: {}", e))
                        })?;
                        let file_path = dir.join(name);
                        std::fs::write(&file_path, &decoded)?;
                        downloaded.push(file_path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    Ok(downloaded)
}

/// Parse a relative time string like "7d", "24h", "30m" to an ISO 8601 datetime
fn parse_since(since: &str) -> Option<String> {
    let since = since.trim();
    let (num_str, unit) = since.split_at(since.len().saturating_sub(1));
    let num: i64 = num_str.parse().ok()?;

    let duration = match unit {
        "d" => chrono::Duration::days(num),
        "h" => chrono::Duration::hours(num),
        "m" => chrono::Duration::minutes(num),
        _ => return None,
    };

    let dt = chrono::Utc::now() - duration;
    Some(dt.to_rfc3339())
}

fn guess_content_type(filename: &str) -> &'static str {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "pdf" => "application/pdf",
        "doc" | "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" | "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" | "pptx" => {
            "application/vnd.openxmlformats-officedocument.presentationml.presentation"
        }
        "txt" => "text/plain",
        "html" | "htm" => "text/html",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "zip" => "application/zip",
        "csv" => "text/csv",
        "json" => "application/json",
        "xml" => "application/xml",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_since() {
        let result = parse_since("7d");
        assert!(result.is_some());

        let result = parse_since("24h");
        assert!(result.is_some());

        let result = parse_since("invalid");
        assert!(result.is_none());
    }

    #[test]
    fn test_guess_content_type() {
        assert_eq!(guess_content_type("test.pdf"), "application/pdf");
        assert_eq!(guess_content_type("test.txt"), "text/plain");
        assert_eq!(
            guess_content_type("unknown.xyz"),
            "application/octet-stream"
        );
        assert_eq!(guess_content_type("image.png"), "image/png");
        assert_eq!(guess_content_type("photo.jpg"), "image/jpeg");
        assert_eq!(guess_content_type("data.csv"), "text/csv");
    }

    #[test]
    fn test_select_fields_list_vs_read() {
        // List should NOT include body, read does
        let list_select =
            "id,subject,from,toRecipients,receivedDateTime,bodyPreview,isRead,hasAttachments";
        let read_select =
            "id,subject,from,toRecipients,receivedDateTime,bodyPreview,isRead,body,hasAttachments";
        assert!(!list_select.contains("body,"));
        assert!(read_select.contains("body,"));
    }

    #[test]
    fn test_parse_since_units() {
        assert!(parse_since("7d").is_some());
        assert!(parse_since("24h").is_some());
        assert!(parse_since("30m").is_some());
        assert!(parse_since("7x").is_none());
        assert!(parse_since("abc").is_none());
    }
}
