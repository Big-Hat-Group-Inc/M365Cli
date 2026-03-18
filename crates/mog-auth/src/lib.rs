//! Authentication and identity management for `mog`.
//!
//! Implements device-code, client-credentials, managed-identity, and federated
//! identity flows against Microsoft Entra ID. Manages named profiles
//! ([`ProfileStore`]), a file-locked token cache ([`TokenCache`]), and
//! least-privilege scope resolution ([`ScopeBundleMapper`]).

pub mod flows;
pub mod profiles;
pub mod scopes;
pub mod token;

pub use profiles::{AuthStrategy, Profile, ProfileStore};
pub use scopes::ScopeBundleMapper;
pub use token::TokenCache;
