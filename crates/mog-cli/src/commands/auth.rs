use crate::{AuthCommands, Cli, ProfileCommands};
use mog_auth::flows;
use mog_auth::profiles::{AuthStrategy, Profile, ProfileStore, DEFAULT_CLIENT_ID};
use mog_auth::scopes::ScopeBundleMapper;
use mog_auth::token::TokenCache;
use mog_core::error::MogError;
use mog_core::output::{OutputFormat, OutputRenderer};
use mog_graph::cloud::Cloud;
use serde_json::json;
use std::io::Read;

pub async fn run(cli: &Cli, command: &AuthCommands, format: OutputFormat) -> Result<(), MogError> {
    match command {
        AuthCommands::Login {
            services,
            readonly,
            strategy,
            auth_type,
            certificate_file,
            certificate_format,
            client_secret_stdin,
            federated_token_file,
            add_scopes: _,
        } => {
            let store = ProfileStore::new();
            let cache = TokenCache::new();

            // Resolve profile
            let profile_name = cli.profile.as_deref().unwrap_or("default");

            // Resolve or create profile
            let profile = match store.get_profile(profile_name) {
                Ok(p) => p,
                Err(_) => {
                    let tenant = cli.tenant.as_deref().unwrap_or("common");
                    let client = cli.client_id.as_deref().unwrap_or(DEFAULT_CLIENT_ID);
                    let strat = strategy.as_deref()
                        .unwrap_or("device-code")
                        .parse::<AuthStrategy>()
                        .map_err(MogError::Validation)?;

                    let scope_bundles = if let Some(svcs) = services {
                        ScopeBundleMapper::resolve_services(svcs, *readonly)
                    } else {
                        vec!["mail:read".to_string(), "calendar:basic".to_string()]
                    };

                    let p = Profile {
                        tenant_id: tenant.to_string(),
                        client_id: client.to_string(),
                        auth_strategy: strat,
                        cloud: Cloud::Public,
                        scope_bundles,
                        api_version: cli.api_version.clone(),
                        certificate_path: certificate_file.clone(),
                        certificate_format: certificate_format.clone(),
                    };
                    store.upsert_profile(profile_name, p.clone())?;
                    eprintln!("Created profile '{}'", profile_name);
                    p
                }
            };

            // Resolve scopes
            let mut scope_bundles = profile.scope_bundles.clone();
            if let Some(svcs) = services {
                scope_bundles = ScopeBundleMapper::resolve_services(svcs, *readonly);
            }
            let scopes = ScopeBundleMapper::expand_bundles(&scope_bundles);

            // Determine auth strategy
            let effective_strategy = if let Some(s) = strategy {
                s.parse::<AuthStrategy>()
                    .map_err(MogError::Validation)?
            } else {
                profile.auth_strategy.clone()
            };

            let tokens = match auth_type.as_str() {
                "app" => {
                    if let Some(token_file) = federated_token_file {
                        flows::federated_identity_flow(
                            &profile.client_id,
                            &profile.tenant_id,
                            token_file,
                            profile.cloud,
                        ).await?
                    } else if *client_secret_stdin {
                        eprintln!("Reading client secret from stdin...");
                        let mut secret = String::new();
                        std::io::stdin().read_to_string(&mut secret)
                            .map_err(|e| MogError::Auth(format!("Failed to read secret from stdin: {}", e)))?;
                        let secret = secret.trim().to_string();
                        flows::client_credentials_flow(
                            &profile.client_id,
                            &profile.tenant_id,
                            &secret,
                            profile.cloud,
                        ).await?
                    } else {
                        return Err(MogError::Validation(
                            "App auth requires --certificate-file, --client-secret-stdin, or --federated-token-file".into()
                        ));
                    }
                }
                "managed-identity" => {
                    flows::managed_identity_flow(
                        cli.client_id.as_deref(),
                        profile.cloud,
                    ).await?
                }
                "federated" => {
                    let token_file = federated_token_file.as_deref()
                        .ok_or_else(|| MogError::Validation("--federated-token-file is required for federated auth".into()))?;
                    flows::federated_identity_flow(
                        &profile.client_id,
                        &profile.tenant_id,
                        token_file,
                        profile.cloud,
                    ).await?
                }
                _ => {
                    // Delegated (user) auth
                    match effective_strategy {
                        AuthStrategy::DeviceCode => {
                            flows::device_code_flow(
                                &profile.client_id,
                                &profile.tenant_id,
                                &scopes,
                                profile.cloud,
                            ).await?
                        }
                        AuthStrategy::Browser => {
                            // For MVP, redirect to device code with a note
                            eprintln!("Note: Browser PKCE flow not yet implemented. Using device code flow.");
                            flows::device_code_flow(
                                &profile.client_id,
                                &profile.tenant_id,
                                &scopes,
                                profile.cloud,
                            ).await?
                        }
                        _ => {
                            return Err(MogError::Validation(format!(
                                "Strategy '{}' requires --auth-type app or federated", effective_strategy
                            )));
                        }
                    }
                }
            };

            cache.set(profile_name, tokens)?;
            eprintln!("Logged in as profile '{}'", profile_name);

            let result = json!({
                "status": "logged_in",
                "profile": profile_name,
                "tenant": profile.tenant_id,
                "strategy": profile.auth_strategy.to_string(),
                "scopes": scope_bundles,
            });
            OutputRenderer::render_value(format, &result)?;
            Ok(())
        }

        AuthCommands::Logout => {
            let store = ProfileStore::new();
            let cache = TokenCache::new();
            let profile_name = store.resolve_profile_name(cli.profile.as_deref())?;
            cache.remove(&profile_name)?;
            eprintln!("Logged out of profile '{}'", profile_name);
            Ok(())
        }

        AuthCommands::Status => {
            let store = ProfileStore::new();
            let cache = TokenCache::new();
            let profile_name = match store.resolve_profile_name(cli.profile.as_deref()) {
                Ok(name) => name,
                Err(_) => {
                    eprintln!("No profile configured. Run 'mog auth login' to get started.");
                    return Ok(());
                }
            };

            let profile = store.get_profile(&profile_name)?;
            let has_tokens = cache.has_valid_tokens(&profile_name);

            let status = json!({
                "profile": profile_name,
                "tenant": profile.tenant_id,
                "clientId": profile.client_id,
                "cloud": profile.cloud.to_string(),
                "authStrategy": profile.auth_strategy.to_string(),
                "scopeBundles": profile.scope_bundles,
                "apiVersion": profile.api_version,
                "authenticated": has_tokens,
            });

            if let Some(tokens) = cache.get_for_refresh(&profile_name) {
                let mut s = status.clone();
                if let Some(obj) = s.as_object_mut() {
                    let expires = chrono::DateTime::from_timestamp(tokens.expires_at, 0)
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_else(|| "unknown".to_string());
                    obj.insert("tokenExpires".to_string(), json!(expires));
                    obj.insert("tokenScopes".to_string(), json!(tokens.scopes));
                    if let Some(account) = &tokens.account {
                        obj.insert("account".to_string(), json!(account));
                    }
                }
                OutputRenderer::render_value(format, &s)?;
            } else {
                OutputRenderer::render_value(format, &status)?;
            }
            Ok(())
        }

        AuthCommands::Profile { command } => {
            run_profile_command(cli, command, format).await
        }

        AuthCommands::ExplainPermissions { command } => {
            let cmd_str = command.join(" ");
            match ScopeBundleMapper::explain_permissions(&cmd_str) {
                Some(explanation) => {
                    if format == OutputFormat::Json {
                        let scopes = ScopeBundleMapper::command_scopes();
                        let required = scopes.get(cmd_str.as_str()).cloned().unwrap_or_default();
                        let result = json!({
                            "command": cmd_str,
                            "requiredScopes": required,
                            "explanation": explanation,
                        });
                        OutputRenderer::render_value(format, &result)?;
                    } else {
                        println!("{}", explanation);
                    }
                    Ok(())
                }
                None => Err(MogError::NotFound(format!("Unknown command: {}", cmd_str))),
            }
        }

        AuthCommands::AdminConsentUrl => {
            let store = ProfileStore::new();
            let profile_name = store.resolve_profile_name(cli.profile.as_deref())?;
            let profile = store.get_profile(&profile_name)?;
            let scopes = ScopeBundleMapper::expand_bundles(&profile.scope_bundles);
            let url = ScopeBundleMapper::admin_consent_url(
                &profile.tenant_id,
                &profile.client_id,
                &scopes,
            );
            if format == OutputFormat::Json {
                let result = json!({"url": url});
                OutputRenderer::render_value(format, &result)?;
            } else {
                println!("{}", url);
            }
            Ok(())
        }
    }
}

async fn run_profile_command(
    _cli: &Cli,
    command: &ProfileCommands,
    format: OutputFormat,
) -> Result<(), MogError> {
    let store = ProfileStore::new();

    match command {
        ProfileCommands::List => {
            let profiles = store.list_profiles();
            let items: Vec<serde_json::Value> = profiles.iter().map(|(name, profile, is_default)| {
                json!({
                    "name": name,
                    "tenant": &profile.tenant_id,
                    "cloud": profile.cloud.to_string(),
                    "strategy": profile.auth_strategy.to_string(),
                    "default": is_default,
                })
            }).collect();

            if items.is_empty() {
                eprintln!("No profiles configured. Run 'mog auth profile create' to create one.");
            } else {
                OutputRenderer::render_value(format, &serde_json::Value::Array(items))?;
            }
            Ok(())
        }

        ProfileCommands::Create { name, tenant, client_id, strategy, cloud, scopes } => {
            let client = client_id.as_deref().unwrap_or(DEFAULT_CLIENT_ID);
            let strat = strategy.parse::<AuthStrategy>()
                .map_err(MogError::Validation)?;
            let cloud_env = cloud.parse::<Cloud>()
                .map_err(MogError::Validation)?;

            let profile = Profile {
                tenant_id: tenant.clone(),
                client_id: client.to_string(),
                auth_strategy: strat,
                cloud: cloud_env,
                scope_bundles: scopes.clone().unwrap_or_default(),
                api_version: "v1.0".into(),
                certificate_path: None,
                certificate_format: None,
            };

            store.upsert_profile(name, profile)?;
            eprintln!("Profile '{}' created.", name);
            Ok(())
        }

        ProfileCommands::Update { name, tenant, client_id, strategy, cloud } => {
            let mut profile = store.get_profile(name)?;
            if let Some(t) = tenant {
                profile.tenant_id = t.clone();
            }
            if let Some(c) = client_id {
                profile.client_id = c.clone();
            }
            if let Some(s) = strategy {
                profile.auth_strategy = s.parse::<AuthStrategy>()
                    .map_err(MogError::Validation)?;
            }
            if let Some(c) = cloud {
                profile.cloud = c.parse::<Cloud>()
                    .map_err(MogError::Validation)?;
            }
            store.upsert_profile(name, profile)?;
            eprintln!("Profile '{}' updated.", name);
            Ok(())
        }

        ProfileCommands::Delete { name } => {
            let cache = TokenCache::new();
            cache.remove(name)?;
            store.delete_profile(name)?;
            eprintln!("Profile '{}' deleted.", name);
            Ok(())
        }

        ProfileCommands::SetDefault { name } => {
            store.set_default(name)?;
            eprintln!("Default profile set to '{}'.", name);
            Ok(())
        }
    }
}
