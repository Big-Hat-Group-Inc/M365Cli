pub mod auth;
pub mod calendar;
pub mod config;
pub mod files;
pub mod graph;
pub mod mail;

use mog_auth::profiles::ProfileStore;
use mog_auth::token::TokenCache;
use mog_core::error::MogError;
use mog_graph::client::GraphClient;

/// Build a GraphClient from the current CLI context
pub fn build_graph_client(
    profile_name: Option<&str>,
    api_version: &str,
    trace: bool,
    top: Option<u32>,
) -> Result<GraphClient, MogError> {
    let store = ProfileStore::new();
    let cache = TokenCache::new();

    let profile_name = store.resolve_profile_name(profile_name)?;
    let profile = store.get_profile(&profile_name)?;

    // Get cached tokens
    let tokens = cache.get(&profile_name)
        .ok_or_else(|| MogError::Auth(format!(
            "No valid tokens for profile '{}'. Run: mog auth login --profile {}",
            profile_name, profile_name
        )))?;

    let access_token = tokens.access_token.clone();
    let _cloud = profile.cloud;
    drop(tokens);

    let config = mog_core::ConfigStore::new().load();

    GraphClient::new(
        access_token,
        profile.cloud,
        api_version,
        config.graph.max_retries,
        top.unwrap_or(config.graph.max_results),
        trace,
    )
}
