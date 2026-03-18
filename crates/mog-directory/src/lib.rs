//! Azure AD directory operations.
//!
//! List, search, and read users and groups in the organization's directory
//! via Microsoft Graph. Supports advanced queries with `ConsistencyLevel:
//! eventual` for `$search` and `$count`.

use mog_core::error::MogError;
use mog_graph::client::{GraphClient, RequestOptions};
use serde_json::Value;
use std::collections::HashMap;

const DEFAULT_USER_SELECT: &str = "id,displayName,mail,jobTitle,department,officeLocation";
const DEFAULT_GROUP_SELECT: &str = "id,displayName,mail,groupTypes,membershipRule";

/// Build request options with ConsistencyLevel: eventual header and $count=true
/// Required for $search queries on directory resources
fn advanced_query_options(params: HashMap<String, String>) -> RequestOptions {
    let mut headers = HashMap::new();
    headers.insert("ConsistencyLevel".to_string(), "eventual".to_string());

    let mut params = params;
    params.insert("$count".to_string(), "true".to_string());

    RequestOptions {
        query_params: params,
        headers,
        ..Default::default()
    }
}

/// List users in the organization
pub async fn list_users(
    client: &GraphClient,
    top: Option<u32>,
    filter: Option<&str>,
) -> Result<Vec<Value>, MogError> {
    let mut params = HashMap::new();
    params.insert("$select".to_string(), DEFAULT_USER_SELECT.to_string());

    if let Some(f) = filter {
        params.insert("$filter".to_string(), f.to_string());
    }

    let options = RequestOptions {
        query_params: params,
        ..Default::default()
    };

    client.get_collection("users", &options, top, false).await
}

/// Search users using $search with ConsistencyLevel: eventual
pub async fn search_users(
    client: &GraphClient,
    query: &str,
    top: Option<u32>,
) -> Result<Vec<Value>, MogError> {
    let mut params = HashMap::new();
    params.insert("$select".to_string(), DEFAULT_USER_SELECT.to_string());
    params.insert("$search".to_string(), format!("\"displayName:{}\"", query));

    let options = advanced_query_options(params);
    client.get_collection("users", &options, top, false).await
}

/// Read a single user by ID or UPN
pub async fn read_user(client: &GraphClient, user_id: &str) -> Result<Value, MogError> {
    let path = format!("users/{}", user_id);
    let mut params = HashMap::new();
    params.insert("$select".to_string(), DEFAULT_USER_SELECT.to_string());

    let options = RequestOptions {
        query_params: params,
        ..Default::default()
    };

    client.request(reqwest::Method::GET, &path, &options).await
}

/// List groups in the organization
pub async fn list_groups(
    client: &GraphClient,
    top: Option<u32>,
    filter: Option<&str>,
) -> Result<Vec<Value>, MogError> {
    let mut params = HashMap::new();
    params.insert("$select".to_string(), DEFAULT_GROUP_SELECT.to_string());

    if let Some(f) = filter {
        params.insert("$filter".to_string(), f.to_string());
    }

    let options = RequestOptions {
        query_params: params,
        ..Default::default()
    };

    client.get_collection("groups", &options, top, false).await
}

/// Search groups using $search with ConsistencyLevel: eventual
pub async fn search_groups(
    client: &GraphClient,
    query: &str,
    top: Option<u32>,
) -> Result<Vec<Value>, MogError> {
    let mut params = HashMap::new();
    params.insert("$select".to_string(), DEFAULT_GROUP_SELECT.to_string());
    params.insert("$search".to_string(), format!("\"displayName:{}\"", query));

    let options = advanced_query_options(params);
    client.get_collection("groups", &options, top, false).await
}

/// Read a single group by ID
pub async fn read_group(client: &GraphClient, group_id: &str) -> Result<Value, MogError> {
    let path = format!("groups/{}", group_id);
    let mut params = HashMap::new();
    params.insert("$select".to_string(), DEFAULT_GROUP_SELECT.to_string());

    let options = RequestOptions {
        query_params: params,
        ..Default::default()
    };

    client.request(reqwest::Method::GET, &path, &options).await
}

/// List members of a group
pub async fn list_group_members(
    client: &GraphClient,
    group_id: &str,
) -> Result<Vec<Value>, MogError> {
    let path = format!("groups/{}/members", group_id);
    let options = RequestOptions::default();
    client.get_collection(&path, &options, None, false).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_user_select_fields() {
        assert!(DEFAULT_USER_SELECT.contains("displayName"));
        assert!(DEFAULT_USER_SELECT.contains("mail"));
        assert!(DEFAULT_USER_SELECT.contains("jobTitle"));
        assert!(DEFAULT_USER_SELECT.contains("department"));
        assert!(DEFAULT_USER_SELECT.contains("officeLocation"));
    }

    #[test]
    fn test_default_group_select_fields() {
        assert!(DEFAULT_GROUP_SELECT.contains("displayName"));
        assert!(DEFAULT_GROUP_SELECT.contains("mail"));
        assert!(DEFAULT_GROUP_SELECT.contains("groupTypes"));
        assert!(DEFAULT_GROUP_SELECT.contains("membershipRule"));
    }

    #[test]
    fn test_advanced_query_options() {
        let params = HashMap::new();
        let opts = advanced_query_options(params);
        assert_eq!(opts.headers.get("ConsistencyLevel").unwrap(), "eventual");
        assert_eq!(opts.query_params.get("$count").unwrap(), "true");
    }

    #[test]
    fn test_search_query_format() {
        let query = "John";
        let search = format!("\"displayName:{}\"", query);
        assert_eq!(search, "\"displayName:John\"");
    }
}
