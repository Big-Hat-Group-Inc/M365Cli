# Tasks: Security Hardening & Production Readiness

## Phase 2: Critical Security Fixes

- [x] 2a. Percent-encode URL parameters in `mog-files` search/export and `mog-auth` scopes
- [x] 2b. Zeroize client secrets in `mog-cli` auth command
- [x] 2c. Path traversal hardening using `Path::components()` in `mog-files`
- [x] 2d. Custom `Debug` impls for `CachedTokens`, `TokenResponse`, `TokenCacheData`

## Phase 3: Stability & Cleanup

- [x] 3a. Replace deprecated `atty` with `std::io::IsTerminal`
- [x] 3b. Replace blocking `std::fs` with `tokio::fs` in async contexts
- [x] 3c. HTTP connection pooling in `GraphClient`
- [x] 3d. Remove unused dependencies (`oauth2`, `aes-gcm`)
- [x] 3e. Named constants for magic numbers

## Phase 4: Observability

- [x] 4a. Structured logging for JSON parse failures (config, token cache, profiles)

## Verification

- [x] `cargo build` compiles cleanly
- [x] `cargo test --workspace` passes (105 tests)
- [x] `cargo clippy --workspace -- -D warnings` clean
- [x] `atty`, `oauth2`, `aes-gcm` absent from `Cargo.lock`
