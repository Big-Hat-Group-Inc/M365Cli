# Rust Project Code Review: mog CLI

## Executive Summary
This code review evaluates the **mog** project — a cross-platform CLI for Microsoft 365 via Microsoft Graph API. The codebase demonstrates **solid architectural patterns** with a 7-crate workspace structure, good separation of concerns, and mature error handling. However, several **critical security**, **stability**, and **best practice** concerns require immediate attention.

**Overall Rating:** 7.5/10 — Good foundation with room for improvement in security hardening and API stability.

---

## Architecture & Design Patterns ✅

### Strengths
- **Well-organized workspace**: 7 crates with clear boundaries (core, auth, graph, mail, calendar, files, cli)
- **Centralized dependency management**: Proper use of `[workspace.dependencies]` in root `Cargo.toml`
- **Error handling strategy**: Custom `MogError` enum with `thiserror` deriving `#[from]` for automatic conversion
- **Exit codes**: Proper Unix exit codes following CLI conventions (130 for SIGINT)
- **Modular design**: Commands split into logical modules with shared `build_graph_client()` helper

### Recommendations
1. **Add crate-level documentation**: Each crate should have a `#![doc = include_str!("../README.md")]` or crate-level docs
2. **Feature flags**: Consider adding feature flags for different auth strategies to reduce binary size
3. **Workspace linting**: Add `[workspace.lints]` for consistent clippy rules across crates

---

## Security Assessment ⚠️ CRITICAL

### Critical Issues

#### 1. **Token Storage in Plain JSON (HIGH)**
**Location:** `crates/mog-auth/src/token.rs:71`

```rust
fn cache_path(&self) -> PathBuf {
    self.dir.join("token_cache.json")  // Plain JSON file
}
```

**Problem:** Access tokens and refresh tokens are stored in **unencrypted JSON** despite file permissions being set to 0600. This is insufficient for production use.

**Recommendation:**
- Use the `keyring` crate (already in dependencies) to store tokens in OS credential stores
- Implement AES-256-GCM encryption at minimum for the token cache file
- Add migration path from file-based to keyring-based storage

#### 2. **Client Secret in Memory Without Zeroization (MEDIUM)**
**Location:** `crates/mog-auth/src/flows.rs:94-99`

```rust
let mut secret = String::new();
std::io::stdin().read_to_string(&mut secret)?;
let secret = secret.trim().to_string();  // Not zeroized
```

**Problem:** Client secrets read from stdin are not securely cleared from memory after use. The `CachedTokens` struct implements `Zeroize` but the flows don't use it for secrets.

**Recommendation:**
```rust
use zeroize::Zeroizing;
let secret = Zeroizing::new(secret.trim().to_string());
```

#### 3. **Path Traversal in File Upload (MEDIUM)**
**Location:** `crates/mog-files/src/lib.rs:10-42`

**Problem:** The `validate_dest_path` function blocks `..` but doesn't prevent all path traversal attacks. The check happens **after** path normalization which could allow bypasses.

**Recommendation:**
- Use `std::path::Path::components()` for robust path validation
- Canonicalize paths before validation
- Add tests for edge cases like `....//`, `%2e%2e`, etc.

#### 4. **URL Construction Without Encoding (MEDIUM)**
**Location:** `crates/mog-files/src/lib.rs:59`

```rust
let path = format!("me/drive/root/search(q='{}')", query);
```

**Problem:** Search queries are directly interpolated into URL without percent-encoding, allowing injection of special characters.

**Recommendation:**
```rust
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
let encoded = utf8_percent_encode(query, NON_ALPHANUMERIC);
let path = format!("me/drive/root/search(q='{}')", encoded);
```

#### 5. **Debug Profile Contains Tokens (LOW)**
**Location:** `crates/mog-auth/src/token.rs:9-20`

**Problem:** `CachedTokens` derives `Debug` which will print tokens in log messages if `tracing` is misconfigured.

**Recommendation:**
```rust
#[derive(Debug)]
pub struct CachedTokens {
    #[debug(skip)]  // Or implement custom Debug
    pub access_token: String,
    // ...
}
```

---

## Stability & Reliability

### Issues

#### 1. **Unwrap on JSON Parsing (MEDIUM)**
**Location:** `crates/mog-core/src/config.rs:189`, `crates/mog-auth/src/token.rs:115`

```rust
serde_json::from_str(&content).unwrap_or_default()
```

**Problem:** Silently ignoring parse errors makes debugging difficult. Corrupted config files are silently replaced with defaults.

**Recommendation:**
```rust
match serde_json::from_str(&content) {
    Ok(config) => config,
    Err(e) => {
        tracing::warn!("Failed to parse config: {}, using defaults", e);
        GlobalConfig::default()
    }
}
```

#### 2. **Missing Request Timeouts in Upload (MEDIUM)**
**Location:** `crates/mog-graph/src/client.rs:372-382`

The `upload_bytes` method uses the default timeout but large file uploads may exceed it.

**Recommendation:** Use the already-configured `upload_timeout` consistently.

#### 3. **No Connection Pooling Configuration (LOW)**
**Location:** `crates/mog-graph/src/client.rs:63-67`

The `reqwest::Client` is built with default settings, missing HTTP/2 and connection pool tuning.

**Recommendation:**
```rust
let http = Client::builder()
    .timeout(Duration::from_secs(timeout_seconds))
    .connect_timeout(Duration::from_secs(connect_timeout_seconds))
    .http2_prior_knowledge()  // Graph supports HTTP/2
    .pool_max_idle_per_host(10)
    .build()
    .map_err(|e| MogError::Network(format!("Failed to create HTTP client: {}", e)))?;
```

#### 4. **Blocking File Operations in Async Context (MEDIUM)**
**Location:** `crates/mog-files/src/lib.rs:116-117`

```rust
let file_data = std::fs::read(local_file)  // Blocking I/O!
    .map_err(|e| MogError::General(format!("...", e)))?;
```

**Problem:** Reading large files blocks the async runtime thread.

**Recommendation:**
```rust
use tokio::fs;
let file_data = fs::read(local_file).await
    .map_err(|e| MogError::General(format!("...", e)))?;
```

---

## Code Quality & Best Practices

### Positive Examples ✅

1. **Proper Serde Field Renaming:** Consistent use of `#[serde(rename = "camelCase")]`
2. **Comprehensive Testing:** All modules have `#[cfg(test)]` sections with good coverage
3. **Tracing Integration:** Proper use of `tracing` for structured logging
4. **File Locking:** Token cache uses `fs2` for proper concurrent access control
5. **Permission Setting:** Unix file permissions set to 0o600 for sensitive files

### Areas for Improvement ⚠️

#### 1. **Magic Numbers**
**Location:** Multiple files

```rust
// In token.rs
now >= self.expires_at - 300  // What is 300?

// In client.rs
if all && results.len() > 1000  // Why 1000?
```

**Recommendation:** Define constants with descriptive names.

#### 2. **Inconsistent Error Messages**
Some errors include context, others don't:

```rust
// Good
return Err(MogError::Network(format!(
    "Cannot reach Microsoft Graph after {} retries. Check network connectivity. Error: {}",
    self.retry_config.max_retries, e
)));

// Less helpful
return Err(MogError::General(format!("HTTP {} — {}", status.as_u16(), ...)));
```

#### 3. **Missing Documentation**
Many public functions lack doc comments. Example in `crates/mog-mail/src/lib.rs`:

```rust
/// Parse a relative time string like "7d", "24h", "30m" to an ISO 8601 datetime
fn parse_since(since: &str) -> Option<String> {
```

This should be `pub` with documentation if intended for external use.

#### 4. **Stringly-Typed Values**
**Location:** `crates/mog-auth/src/scopes.rs:150-153`

```rust
.replace(' ', "%20")
.replace(':', "%3A")
.replace('/', "%2F")
```

**Recommendation:** Use `percent-encoding` crate instead of manual replacement.

---

## Dependency Review

### Unused Dependencies
Run `cargo udeps` to identify:
- `oauth2 = "5.0.0-rc.1"` — Listed but not imported anywhere
- `aes-gcm` — Listed but not used (would be needed for token encryption)

### Pre-Release Dependencies (WARNING)
- `oauth2 = "5.0.0-rc.1"` — Release candidate, not stable

### Security Advisories
Recommend running:
```bash
cargo install cargo-audit
cargo audit
```

### Dependency Conflicts
- `atty = "0.2"` is deprecated; consider `std::io::IsTerminal` (Rust 1.70+)

---

## Performance Considerations

### Memory Efficiency

1. **Token Cloning:** `CachedTokens::get()` clones the entire struct including tokens. Consider `Arc<CachedTokens>` for shared access.

2. **Base64 Encoding:** Mail attachments are fully loaded into memory before encoding. For large attachments, consider streaming.

3. **Response Buffering:** `request_bytes` loads entire response into memory. Large file downloads could benefit from streaming to disk.

---

## Testing

### Strengths
- Good unit test coverage in all crates
- Tests use temp directories for isolation
- Edge cases covered (empty files, parse errors)

### Gaps
1. **No integration tests** — No tests that actually call Graph API (mock server needed)
2. **No property-based testing** — QuickCheck could find edge cases in URL parsing
3. **Missing async tests** — Most tests are synchronous

**Recommendation:** Add `tokio::test` for async test coverage:
```rust
#[tokio::test]
async fn test_graph_request() {
    // Test with mock server
}
```

---

## CI/CD Recommendations

Add `.github/workflows/ci.yml`:

```yaml
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --workspace
      - run: cargo clippy --workspace -- -D warnings
      - run: cargo audit
      - run: cargo fmt --check
```

---

## Action Items (Prioritized)

### Critical (Fix Immediately)
1. [ ] **Encrypt token cache** or use OS keyring
2. [ ] **Zeroize client secrets** from memory
3. [ ] **Percent-encode URL parameters** in file search

### High Priority
4. [ ] **Replace blocking I/O** with `tokio::fs` in async contexts
5. [ ] **Add structured logging** for token operations
6. [ ] **Fix path traversal** validation to use `Path::components()`

### Medium Priority
7. [ ] **Add HTTP/2 support** and connection pooling
8. [ ] **Implement custom Debug** for sensitive types
9. [ ] **Replace `atty`** with `std::io::IsTerminal`
10. [ ] **Remove or use** `oauth2` pre-release dependency

### Low Priority
11. [ ] **Add crate documentation**
12. [ ] **Feature flags** for auth strategies
13. [ ] **Integration tests** with mock Graph API

---

## Conclusion

The **mog** CLI demonstrates solid Rust fundamentals with good architectural decisions. The primary concerns center around **security hardening** — particularly token storage and memory safety for credentials. Addressing the critical items would elevate this from a development/MVP tool to production-ready software.

The codebase shows evidence of experienced Rust developers (proper error handling, workspace structure, testing), but would benefit from a security-focused review pass before handling production credentials.
