use mog_core::error::MogError;
use mog_graph::client::{GraphClient, RequestOptions};
use mog_graph::upload::{self, UploadSession};
use reqwest::Method;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;

/// Search drive files
pub async fn search_files(
    client: &GraphClient,
    query: &str,
    top: Option<u32>,
) -> Result<Vec<Value>, MogError> {
    let mut params = HashMap::new();
    if let Some(t) = top {
        params.insert("$top".to_string(), t.to_string());
    }
    params.insert(
        "$select".to_string(),
        "id,name,size,lastModifiedDateTime,webUrl,file,folder".to_string(),
    );

    let path = format!("me/drive/root/search(q='{}')", query);
    let options = RequestOptions {
        query_params: params,
        ..Default::default()
    };

    client.get_collection(&path, &options, top, false).await
}

/// Download a file by item ID
pub async fn download_file(
    client: &GraphClient,
    item_id: &str,
    out_path: &str,
) -> Result<String, MogError> {
    let path = format!("me/drive/items/{}/content", item_id);
    let bytes = client.request_bytes(Method::GET, &path, &RequestOptions::default()).await?;

    let out = Path::new(out_path);
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(out, &bytes)?;

    Ok(out_path.to_string())
}

/// Export/convert a file (e.g., to PDF)
pub async fn export_file(
    client: &GraphClient,
    item_id: &str,
    format: &str,
    out_path: &str,
) -> Result<String, MogError> {
    let path = format!("me/drive/items/{}/content?format={}", item_id, format);
    let bytes = client.request_bytes(Method::GET, &path, &RequestOptions::default()).await?;

    let out = Path::new(out_path);
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(out, &bytes)?;

    Ok(out_path.to_string())
}

/// Upload a file (simple upload for small files, resumable for large)
pub async fn upload_file(
    client: &GraphClient,
    dest: &str,
    local_file: &str,
    resumable: bool,
    chunk_size: Option<u64>,
) -> Result<Value, MogError> {
    let file_data = std::fs::read(local_file)
        .map_err(|e| MogError::General(format!("Failed to read file '{}': {}", local_file, e)))?;
    let file_size = file_data.len() as u64;

    // Use resumable upload for files > 4MB or when explicitly requested
    if resumable || file_size > 4 * 1024 * 1024 {
        resumable_upload(client, dest, &file_data, file_size, chunk_size).await
    } else {
        simple_upload(client, dest, &file_data).await
    }
}

/// Simple upload (< 4MB)
async fn simple_upload(
    client: &GraphClient,
    dest: &str,
    data: &[u8],
) -> Result<Value, MogError> {
    let dest = dest.trim_start_matches('/');
    let path = format!("me/drive/root:/{}:/content", dest);

    let options = RequestOptions {
        body_bytes: Some(data.to_vec()),
        ..Default::default()
    };

    client.request(Method::PUT, &path, &options).await
}

/// Resumable upload using upload sessions
async fn resumable_upload(
    client: &GraphClient,
    dest: &str,
    data: &[u8],
    file_size: u64,
    chunk_size: Option<u64>,
) -> Result<Value, MogError> {
    let chunk_size = match chunk_size {
        Some(cs) => upload::validate_chunk_size(cs)?,
        None => upload::DEFAULT_CHUNK_SIZE,
    };

    // Step 1: Create upload session
    let dest = dest.trim_start_matches('/');
    let session_path = format!("me/drive/root:/{}:/createUploadSession", dest);
    let session_body = json!({
        "item": {
            "@microsoft.graph.conflictBehavior": "rename"
        }
    });

    let options = RequestOptions {
        body: Some(session_body),
        ..Default::default()
    };

    let session_response = client.request(Method::POST, &session_path, &options).await?;

    let upload_url = session_response.get("uploadUrl")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MogError::General("No uploadUrl in session response".into()))?
        .to_string();

    eprintln!("Upload session created: {}", &upload_url[..80.min(upload_url.len())]);

    let mut session = UploadSession::new(upload_url.clone(), file_size, chunk_size);

    if let Some(exp) = session_response.get("expirationDateTime").and_then(|v| v.as_str()) {
        session.expiration = Some(exp.to_string());
    }

    // Step 2: Upload chunks
    while !session.is_complete() {
        let start = session.bytes_uploaded as usize;
        let chunk_len = session.next_chunk_size() as usize;
        let end = start + chunk_len;
        let chunk = &data[start..end];

        let content_range = session.content_range();
        let content_length = chunk_len as u64;

        eprintln!(
            "Uploading chunk: {} ({}/{})",
            content_range, session.bytes_uploaded + content_length, file_size
        );

        let response = client.upload_bytes(&upload_url, chunk, &content_range, content_length).await?;
        let status = response.status();

        if status.is_success() {
            let body = response.text().await
                .map_err(|e| MogError::Network(format!("Failed to read upload response: {}", e)))?;

            session.bytes_uploaded += content_length;

            if session.is_complete() || status.as_u16() == 200 || status.as_u16() == 201 {
                // Upload complete
                if body.is_empty() {
                    return Ok(json!({"status": "uploaded", "path": dest}));
                }
                let result: Value = serde_json::from_str(&body)
                    .unwrap_or(json!({"status": "uploaded", "path": dest}));
                return Ok(result);
            }
        } else {
            let body = response.text().await.unwrap_or_default();
            return Err(MogError::General(format!(
                "Upload chunk failed ({}): {}",
                status.as_u16(), body
            )));
        }
    }

    Ok(json!({"status": "uploaded", "path": dest}))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_dest_path_normalization() {
        let dest = "/Shared/report.pdf";
        let trimmed = dest.trim_start_matches('/');
        assert_eq!(trimmed, "Shared/report.pdf");
    }

    #[test]
    fn test_dest_no_leading_slash() {
        let dest = "Documents/file.txt";
        let trimmed = dest.trim_start_matches('/');
        assert_eq!(trimmed, "Documents/file.txt");
    }

    #[test]
    fn test_upload_path_construction() {
        let dest = "Documents/report.pdf";
        let path = format!("me/drive/root:/{}:/content", dest);
        assert_eq!(path, "me/drive/root:/Documents/report.pdf:/content");
    }

    #[test]
    fn test_chunk_size_validation_integration() {
        use mog_graph::upload;
        // Default should work
        assert!(upload::validate_chunk_size(upload::DEFAULT_CHUNK_SIZE).is_ok());
        // Too small
        assert!(upload::validate_chunk_size(100).is_err());
        // Too big
        assert!(upload::validate_chunk_size(upload::MAX_CHUNK_SIZE + 1).is_err());
    }
}
