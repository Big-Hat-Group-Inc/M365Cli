//! People search and relevance operations.
//!
//! Search the organization's people directory and retrieve relevance-ranked
//! contacts for the signed-in user via Microsoft Graph's People API.

use mog_core::error::MogError;
use mog_graph::client::{GraphClient, RequestOptions};
use serde_json::Value;
use std::collections::HashMap;

const DEFAULT_SELECT: &str = "id,displayName,scoredEmailAddresses,jobTitle,department";

/// Search people using $search
pub async fn search_people(
    client: &GraphClient,
    query: &str,
    top: Option<u32>,
) -> Result<Vec<Value>, MogError> {
    let mut params = HashMap::new();
    params.insert("$select".to_string(), DEFAULT_SELECT.to_string());
    params.insert("$search".to_string(), format!("\"{}\"", query));

    let options = RequestOptions {
        query_params: params,
        ..Default::default()
    };

    client
        .get_collection("me/people", &options, top, false)
        .await
}

/// Get relevance-ranked people for the signed-in user
pub async fn relevant_people(
    client: &GraphClient,
    top: Option<u32>,
) -> Result<Vec<Value>, MogError> {
    let mut params = HashMap::new();
    params.insert("$select".to_string(), DEFAULT_SELECT.to_string());

    let options = RequestOptions {
        query_params: params,
        ..Default::default()
    };

    client
        .get_collection("me/people", &options, top, false)
        .await
}

/// Extract the primary scored email address from a person value
pub fn extract_primary_email(person: &Value) -> String {
    person
        .get("scoredEmailAddresses")
        .and_then(|addrs| addrs.as_array())
        .and_then(|arr| arr.first())
        .and_then(|addr| addr.get("address"))
        .and_then(|a| a.as_str())
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_default_select_fields() {
        assert!(DEFAULT_SELECT.contains("displayName"));
        assert!(DEFAULT_SELECT.contains("scoredEmailAddresses"));
        assert!(DEFAULT_SELECT.contains("jobTitle"));
        assert!(DEFAULT_SELECT.contains("department"));
    }

    #[test]
    fn test_extract_primary_email() {
        let person = json!({
            "scoredEmailAddresses": [
                {"address": "jane@example.com", "relevanceScore": 10.0}
            ]
        });
        assert_eq!(extract_primary_email(&person), "jane@example.com");
    }

    #[test]
    fn test_extract_primary_email_empty() {
        let person = json!({"scoredEmailAddresses": []});
        assert_eq!(extract_primary_email(&person), "");
    }

    #[test]
    fn test_extract_primary_email_missing() {
        let person = json!({"displayName": "Jane"});
        assert_eq!(extract_primary_email(&person), "");
    }
}
