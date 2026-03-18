# Tasks

## 1. Profile model and storage

- [x] Implement the profile store schema and default profile selection behavior.
- [x] Implement profile CRUD commands and validation.
- [x] Implement platform-specific storage paths and `MOG_CONFIG_DIR` override support.

## 2. Authentication flows

- [x] Implement device code login.
- [x] Implement browser-based PKCE login.
- [x] Implement client credentials with certificate support.
- [x] Implement managed identity and federated identity flows.

## 3. Secure token handling

- [x] Implement keyring-backed token storage with encrypted-file fallback.
- [x] Implement token refresh, rotation, and invalidation behavior.
- [x] Implement logout and profile deletion cleanup.

## 4. Permission and consent UX

- [x] Implement scope bundle mapping for least-privilege consent.
- [x] Implement missing-scope detection and remediation guidance.
- [x] Implement `explain-permissions` and admin consent URL helpers.

## 5. Verification

- [x] Add unit tests for profile resolution, scope mapping, and cache invalidation.
- [x] Add integration tests for delegated and app-only auth flows.
- [x] Add security tests covering redaction and secret handling.
