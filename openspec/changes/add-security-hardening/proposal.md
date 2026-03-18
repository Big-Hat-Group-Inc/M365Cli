# Change Proposal: Security Hardening & Production Readiness

## Why

A code review identified 13 action items across Critical/High/Medium/Low priorities. Before `mog` handles real credentials and production data, these code-level security and stability issues must be resolved. Unaddressed, they expose users to URL injection, path traversal, credential leakage in logs, and reliability issues from blocking I/O in async contexts.

## What Changes

- Percent-encode user-supplied values interpolated into Graph API URLs (injection fix).
- Harden path traversal validation using `Path::components()` instead of string matching.
- Zeroize client secrets read from stdin after use.
- Replace `#[derive(Debug)]` on token types with manual impls that redact secrets.
- Replace deprecated `atty` crate with `std::io::IsTerminal` (stable since Rust 1.70).
- Replace blocking `std::fs` calls with `tokio::fs` in async functions.
- Add HTTP connection pooling configuration.
- Remove unused workspace dependencies (`oauth2`, `aes-gcm`).
- Extract magic numbers into named constants.
- Add structured `tracing::warn!` logging for JSON parse failures that silently fall back to defaults.

## Scope

In scope:

- All code-level fixes from the code review (Critical, High, Medium, Low)
- New unit tests for security-sensitive changes
- Dependency cleanup

Out of scope:

- Crate-level documentation (separate proposal)
- CI/CD pipeline changes (integration tests, feature flags)
- New features or command changes

## Impact

- **Security**: Eliminates URL injection, path traversal bypass, and credential exposure vectors.
- **Stability**: Removes blocking I/O from async contexts; removes deprecated dependencies.
- **Observability**: Silent parse failures now produce structured warnings.
- **Dependency hygiene**: Removes 2 unused crates, replaces 1 deprecated crate.
