//! Microsoft Graph HTTP transport layer.
//!
//! Provides [`GraphClient`] with automatic retry, exponential back-off,
//! throttle handling, pagination, sovereign-cloud support, and upload session
//! management for the Microsoft Graph API.

pub mod client;
pub mod cloud;
pub mod pagination;
pub mod retry;
pub mod upload;

pub use client::GraphClient;
pub use cloud::{Cloud, CloudEndpoints};
