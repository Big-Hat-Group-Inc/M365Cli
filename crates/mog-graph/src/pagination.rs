use serde_json::Value;

/// Extract the @odata.nextLink from a Graph response
pub fn extract_next_link(body: &Value) -> Option<String> {
    body.get("@odata.nextLink")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Extract the value array from a Graph collection response
pub fn extract_values(body: &Value) -> Vec<Value> {
    body.get("value")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
}

/// Extract @odata.count if present
pub fn extract_count(body: &Value) -> Option<u64> {
    body.get("@odata.count").and_then(|v| v.as_u64())
}

/// Normalize a nextLink URL to be absolute
pub fn normalize_next_link(next_link: &str, base_url: &str) -> String {
    if next_link.starts_with("http://") || next_link.starts_with("https://") {
        next_link.to_string()
    } else {
        format!("{}{}", base_url.trim_end_matches('/'), next_link)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_next_link() {
        let body = json!({
            "value": [],
            "@odata.nextLink": "https://graph.microsoft.com/v1.0/me/messages?$skip=25"
        });
        assert_eq!(
            extract_next_link(&body),
            Some("https://graph.microsoft.com/v1.0/me/messages?$skip=25".to_string())
        );
    }

    #[test]
    fn test_extract_next_link_none() {
        let body = json!({"value": []});
        assert_eq!(extract_next_link(&body), None);
    }

    #[test]
    fn test_extract_values() {
        let body = json!({"value": [{"id": "1"}, {"id": "2"}]});
        assert_eq!(extract_values(&body).len(), 2);
    }

    #[test]
    fn test_normalize_next_link_absolute() {
        let link = "https://graph.microsoft.com/v1.0/me/messages?$skip=25";
        assert_eq!(
            normalize_next_link(link, "https://graph.microsoft.com"),
            link
        );
    }

    #[test]
    fn test_normalize_next_link_relative() {
        let link = "/v1.0/me/messages?$skip=25";
        assert_eq!(
            normalize_next_link(link, "https://graph.microsoft.com"),
            "https://graph.microsoft.com/v1.0/me/messages?$skip=25"
        );
    }

    #[test]
    fn test_normalize_next_link_trailing_slash() {
        let link = "/v1.0/me/messages";
        assert_eq!(
            normalize_next_link(link, "https://graph.microsoft.com/"),
            "https://graph.microsoft.com/v1.0/me/messages"
        );
    }

    #[test]
    fn test_extract_count() {
        let body = json!({"value": [], "@odata.count": 42});
        assert_eq!(extract_count(&body), Some(42));

        let body = json!({"value": []});
        assert_eq!(extract_count(&body), None);
    }

    #[test]
    fn test_extract_values_missing() {
        let body = json!({"noValueKey": true});
        assert_eq!(extract_values(&body).len(), 0);
    }
}
