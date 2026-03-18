# Spec: Security Hardening

## URL Parameter Encoding

**Given** a user searches files with a query containing special characters (`'`, `(`, `)`, spaces, unicode)
**When** the search request is constructed
**Then** the query is percent-encoded before interpolation into the URL path

**Given** a user exports a file with a format parameter containing special characters
**When** the export request is constructed
**Then** the format value is percent-encoded

**Given** the `admin_consent_url` function receives scopes with special characters
**When** the URL is constructed
**Then** scope values are percent-encoded using standard percent-encoding (not a manual replacement function)

## Path Traversal Prevention

**Given** a destination path containing `..` as a path component (e.g., `../../etc/passwd`)
**When** the path is validated
**Then** it is rejected with a validation error

**Given** a destination path containing `..` inside a filename (e.g., `my..file.txt`)
**When** the path is validated
**Then** it is accepted (not a traversal attempt)

**Given** an obfuscated traversal attempt (e.g., `....//`, encoded variants)
**When** the path is validated
**Then** it is rejected with a validation error

## Secret Zeroization

**Given** a client secret is read from stdin for app authentication
**When** the authentication flow completes (success or failure)
**Then** the secret is zeroized in memory before being dropped

## Debug Redaction

**Given** a `CachedTokens` value is formatted with `{:?}`
**Then** the `access_token`, `refresh_token`, and `id_token` fields display as `[REDACTED]`

**Given** a `TokenResponse` value is formatted with `{:?}`
**Then** the `access_token`, `refresh_token`, and `id_token` fields display as `[REDACTED]`

**Given** a `TokenCacheData` value is formatted with `{:?}`
**Then** the output shows the number of cached entries, not individual token values

## Deprecated Dependency Removal

**Given** the project is built
**Then** the `atty` crate is not present in `Cargo.lock`
**And** TTY detection uses `std::io::IsTerminal`

## Async I/O Consistency

**Given** an async function performs file I/O (read, write, create_dir_all)
**Then** it uses `tokio::fs` instead of `std::fs` to avoid blocking the async runtime

## Connection Pooling

**Given** the `GraphClient` HTTP client is constructed
**Then** it configures `pool_max_idle_per_host(10)` for connection reuse

## Unused Dependency Removal

**Given** the workspace is built
**Then** `oauth2` and `aes-gcm` are not present in `Cargo.lock`

## Named Constants

**Given** a numeric literal is used for a domain-meaningful threshold
**Then** it is extracted into a named constant with a descriptive name

## Structured Parse Logging

**Given** a JSON configuration file fails to parse
**When** the fallback to defaults is triggered
**Then** a `tracing::warn!` message is emitted with the file path and parse error
