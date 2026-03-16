# Tasks

## 1. Profile model and storage

- [ ] Implement the profile store schema and default profile selection behavior.
- [ ] Implement profile CRUD commands and validation.
- [ ] Implement platform-specific storage paths and `MOG_CONFIG_DIR` override support.

## 2. Authentication flows

- [ ] Implement device code login.
- [ ] Implement browser-based PKCE login.
- [ ] Implement client credentials with certificate support.
- [ ] Implement managed identity and federated identity flows.

## 3. Secure token handling

- [ ] Implement keyring-backed token storage with encrypted-file fallback.
- [ ] Implement token refresh, rotation, and invalidation behavior.
- [ ] Implement logout and profile deletion cleanup.

## 4. Permission and consent UX

- [ ] Implement scope bundle mapping for least-privilege consent.
- [ ] Implement missing-scope detection and remediation guidance.
- [ ] Implement `explain-permissions` and admin consent URL helpers.

## 5. Verification

- [ ] Add unit tests for profile resolution, scope mapping, and cache invalidation.
- [ ] Add integration tests for delegated and app-only auth flows.
- [ ] Add security tests covering redaction and secret handling.
