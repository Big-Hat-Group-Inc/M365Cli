//! Personal contacts operations.
//!
//! List, search, and read the signed-in user's personal contacts from the
//! Microsoft Graph `/me/contacts` endpoint.

use mog_core::error::MogError;
use mog_graph::client::{GraphClient, RequestOptions};
use serde_json::Value;
use std::collections::HashMap;

const DEFAULT_SELECT: &str = "id,displayName,emailAddresses,mobilePhone,companyName";

/// List the signed-in user's personal contacts
pub async fn list_contacts(
    client: &GraphClient,
    top: Option<u32>,
    filter: Option<&str>,
) -> Result<Vec<Value>, MogError> {
    let mut params = HashMap::new();
    params.insert("$select".to_string(), DEFAULT_SELECT.to_string());
    params.insert("$orderby".to_string(), "displayName".to_string());

    if let Some(f) = filter {
        params.insert("$filter".to_string(), f.to_string());
    }

    let options = RequestOptions {
        query_params: params,
        ..Default::default()
    };

    client
        .get_collection("me/contacts", &options, top, false)
        .await
}

/// Search contacts using $search
pub async fn search_contacts(
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
        .get_collection("me/contacts", &options, top, false)
        .await
}

/// Read a single contact by ID
pub async fn read_contact(client: &GraphClient, contact_id: &str) -> Result<Value, MogError> {
    let path = format!("me/contacts/{}", contact_id);
    client
        .request(reqwest::Method::GET, &path, &RequestOptions::default())
        .await
}

/// Extract the primary email address from a contact value
pub fn extract_primary_email(contact: &Value) -> String {
    contact
        .get("emailAddresses")
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
        assert!(DEFAULT_SELECT.contains("emailAddresses"));
        assert!(DEFAULT_SELECT.contains("mobilePhone"));
        assert!(DEFAULT_SELECT.contains("companyName"));
    }

    #[test]
    fn test_extract_primary_email() {
        let contact = json!({
            "emailAddresses": [
                {"address": "john@example.com", "name": "John"},
                {"address": "john2@example.com", "name": "John Alt"}
            ]
        });
        assert_eq!(extract_primary_email(&contact), "john@example.com");
    }

    #[test]
    fn test_extract_primary_email_empty() {
        let contact = json!({"emailAddresses": []});
        assert_eq!(extract_primary_email(&contact), "");
    }

    #[test]
    fn test_extract_primary_email_missing() {
        let contact = json!({"displayName": "John"});
        assert_eq!(extract_primary_email(&contact), "");
    }
}
