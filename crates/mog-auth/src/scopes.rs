use std::collections::HashMap;

/// Maps human-friendly scope bundle names to Graph permissions
pub struct ScopeBundleMapper;

impl ScopeBundleMapper {
    /// Get delegated scopes for a bundle name
    pub fn delegated_scopes(bundle: &str) -> Option<Vec<&'static str>> {
        match bundle {
            "mail:basic" => Some(vec!["Mail.ReadBasic"]),
            "mail:read" => Some(vec!["Mail.Read"]),
            "mail:send" => Some(vec!["Mail.Send"]),
            "mail" => Some(vec!["Mail.Read", "Mail.Send"]),
            "calendar:basic" => Some(vec!["Calendars.Read"]),
            "calendar:read" => Some(vec!["Calendars.Read"]),
            "calendar" => Some(vec!["Calendars.ReadWrite"]),
            "files:read" => Some(vec!["Files.Read"]),
            "files" => Some(vec!["Files.ReadWrite"]),
            "tasks:read" => Some(vec!["Tasks.Read"]),
            "tasks" => Some(vec!["Tasks.ReadWrite"]),
            "contacts:read" => Some(vec!["Contacts.Read"]),
            "people:read" => Some(vec!["People.Read"]),
            "directory:read" => Some(vec!["User.ReadBasic.All"]),
            "user" => Some(vec!["User.Read"]),
            _ => None,
        }
    }

    /// Get app-only scopes for a bundle name
    pub fn app_scopes(bundle: &str) -> Option<Vec<&'static str>> {
        match bundle {
            "mail:basic" => Some(vec!["Mail.Read.All"]),
            "mail:read" => Some(vec!["Mail.Read"]),
            "mail:send" => Some(vec!["Mail.Send"]),
            "mail" => Some(vec!["Mail.Read", "Mail.Send"]),
            "calendar:basic" => Some(vec!["Calendars.Read"]),
            "calendar:read" => Some(vec!["Calendars.Read"]),
            "calendar" => Some(vec!["Calendars.ReadWrite"]),
            "files:read" => Some(vec!["Files.Read.All"]),
            "files" => Some(vec!["Files.ReadWrite.All"]),
            "tasks:read" => Some(vec!["Tasks.Read.All"]),
            "tasks" => Some(vec!["Tasks.ReadWrite.All"]),
            "contacts:read" => Some(vec!["Contacts.Read"]),
            "people:read" => Some(vec!["People.Read.All"]),
            "directory:read" => Some(vec!["User.Read.All"]),
            _ => None,
        }
    }

    /// Resolve scope bundles from --services and --readonly flags
    pub fn resolve_services(services: &[String], readonly: bool) -> Vec<String> {
        let mut bundles = Vec::new();
        for service in services {
            let bundle = if readonly {
                match service.as_str() {
                    "mail" => "mail:read",
                    "calendar" => "calendar:basic",
                    "files" => "files:read",
                    "tasks" => "tasks:read",
                    "contacts" => "contacts:read",
                    "people" => "people:read",
                    "directory" => "directory:read",
                    other => other,
                }
            } else {
                service.as_str()
            };
            bundles.push(bundle.to_string());
        }
        bundles
    }

    /// Expand scope bundles to actual Graph permission strings (delegated)
    pub fn expand_bundles(bundles: &[String]) -> Vec<String> {
        let mut scopes = vec!["User.Read".to_string()]; // Always included
        let mut seen = std::collections::HashSet::new();
        seen.insert("User.Read".to_string());

        for bundle in bundles {
            if let Some(bundle_scopes) = Self::delegated_scopes(bundle) {
                for scope in bundle_scopes {
                    if seen.insert(scope.to_string()) {
                        scopes.push(scope.to_string());
                    }
                }
            }
        }
        scopes
    }

    /// Get required scopes for a command
    pub fn command_scopes() -> HashMap<&'static str, Vec<&'static str>> {
        let mut map = HashMap::new();
        map.insert("mail list", vec!["Mail.ReadBasic"]);
        map.insert("mail search", vec!["Mail.Read"]);
        map.insert("mail read", vec!["Mail.Read"]);
        map.insert("mail send", vec!["Mail.Send"]);
        map.insert("mail attachments list", vec!["Mail.Read"]);
        map.insert("mail attachments download", vec!["Mail.Read"]);
        map.insert("calendar today", vec!["Calendars.Read"]);
        map.insert("calendar week", vec!["Calendars.Read"]);
        map.insert("calendar list", vec!["Calendars.Read"]);
        map.insert("calendar create", vec!["Calendars.ReadWrite"]);
        map.insert("calendar update", vec!["Calendars.ReadWrite"]);
        map.insert("calendar delete", vec!["Calendars.ReadWrite"]);
        map.insert("files search", vec!["Files.Read"]);
        map.insert("files download", vec!["Files.Read"]);
        map.insert("files export", vec!["Files.Read"]);
        map.insert("files upload", vec!["Files.ReadWrite"]);
        map
    }

    /// Explain what permissions a command needs and why
    pub fn explain_permissions(command: &str) -> Option<String> {
        let scopes = Self::command_scopes();
        let required = scopes.get(command)?;

        let mut explanation = format!("Command '{}' requires the following permissions:\n\n", command);
        for scope in required {
            let reason = match *scope {
                "Mail.ReadBasic" => "List mail message headers (from, to, subject, date)",
                "Mail.Read" => "Read full mail message content including body and attachments",
                "Mail.Send" => "Send mail on behalf of the user",
                "Calendars.Read" => "Read calendar events and free/busy information",
                "Calendars.ReadWrite" => "Create, update, and delete calendar events",
                "Files.Read" => "Read files in OneDrive (search, download, export)",
                "Files.ReadWrite" => "Read and write files in OneDrive (upload)",
                _ => "Required for this operation",
            };
            explanation.push_str(&format!("  • {} — {}\n", scope, reason));
        }
        explanation.push_str(&format!(
            "\nTo add missing scopes: mog auth login --add-scopes {}\n",
            required.join(",")
        ));
        Some(explanation)
    }

    /// Generate admin consent URL
    pub fn admin_consent_url(tenant_id: &str, client_id: &str, scopes: &[String]) -> String {
        let scope_str = scopes.join(" ");
        format!(
            "https://login.microsoftonline.com/{}/adminconsent?client_id={}&scope={}&redirect_uri=http://localhost",
            tenant_id, client_id, urlencoding(&scope_str)
        )
    }
}

fn urlencoding(s: &str) -> String {
    s.replace(' ', "%20")
        .replace(':', "%3A")
        .replace('/', "%2F")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delegated_scopes() {
        assert_eq!(
            ScopeBundleMapper::delegated_scopes("mail:basic"),
            Some(vec!["Mail.ReadBasic"])
        );
        assert_eq!(
            ScopeBundleMapper::delegated_scopes("mail"),
            Some(vec!["Mail.Read", "Mail.Send"])
        );
        assert_eq!(ScopeBundleMapper::delegated_scopes("invalid"), None);
    }

    #[test]
    fn test_resolve_services_readonly() {
        let services = vec!["mail".to_string(), "calendar".to_string()];
        let bundles = ScopeBundleMapper::resolve_services(&services, true);
        assert_eq!(bundles, vec!["mail:read", "calendar:basic"]);
    }

    #[test]
    fn test_resolve_services_readwrite() {
        let services = vec!["mail".to_string()];
        let bundles = ScopeBundleMapper::resolve_services(&services, false);
        assert_eq!(bundles, vec!["mail"]);
    }

    #[test]
    fn test_expand_bundles() {
        let bundles = vec!["mail:read".to_string(), "calendar:basic".to_string()];
        let scopes = ScopeBundleMapper::expand_bundles(&bundles);
        assert!(scopes.contains(&"User.Read".to_string()));
        assert!(scopes.contains(&"Mail.Read".to_string()));
        assert!(scopes.contains(&"Calendars.Read".to_string()));
    }

    #[test]
    fn test_explain_permissions() {
        let explanation = ScopeBundleMapper::explain_permissions("mail send").unwrap();
        assert!(explanation.contains("Mail.Send"));
    }

    #[test]
    fn test_command_scopes_complete() {
        let scopes = ScopeBundleMapper::command_scopes();
        assert!(scopes.contains_key("mail list"));
        assert!(scopes.contains_key("calendar create"));
        assert!(scopes.contains_key("files upload"));
    }

    #[test]
    fn test_app_scopes_differ_from_delegated() {
        // App-only scopes use .All suffix
        let delegated = ScopeBundleMapper::delegated_scopes("files:read").unwrap();
        let app = ScopeBundleMapper::app_scopes("files:read").unwrap();
        assert_eq!(delegated, vec!["Files.Read"]);
        assert_eq!(app, vec!["Files.Read.All"]);
    }

    #[test]
    fn test_expand_bundles_deduplicates() {
        let bundles = vec!["user".to_string(), "mail:read".to_string()];
        let scopes = ScopeBundleMapper::expand_bundles(&bundles);
        // User.Read should appear only once even though it's always added and in "user" bundle
        let count = scopes.iter().filter(|s| *s == "User.Read").count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_admin_consent_url() {
        let url = ScopeBundleMapper::admin_consent_url(
            "tenant-123",
            "client-456",
            &["Mail.Read".to_string(), "User.Read".to_string()],
        );
        assert!(url.contains("tenant-123"));
        assert!(url.contains("client-456"));
        assert!(url.contains("Mail.Read"));
    }
}
