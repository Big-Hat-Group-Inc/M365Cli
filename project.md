# Project: mog — Microsoft 365 CLI

## Status

**Version:** 0.1.0 (Alpha)
**Branch:** 0.5-Alpha
**Rating:** 7.5/10 — Solid foundation, security hardening in progress

## Architecture

### Workspace Crates

| Crate | Role | Dependencies |
|-------|------|-------------|
| `mog-cli` | Binary entry point. Clap command dispatch, logging setup, output format selection | All other crates |
| `mog-core` | Shared foundation: `MogError` with exit codes, `OutputRenderer` (JSON/table/plain/CSV), `ConfigStore` | None (leaf crate) |
| `mog-graph` | Graph HTTP client: `GraphClient`, retry with exponential backoff, `Retry-After` handling, `@odata.nextLink` pagination, upload sessions, sovereign cloud endpoints | `mog-core` |
| `mog-auth` | Auth flows (device code, browser PKCE, client credentials, managed identity, federated), `ProfileStore`, `TokenCache`, `ScopeBundleMapper` | `mog-core`, `mog-graph` |
| `mog-mail` | Mail module: list (headers-only), search (KQL), read, send, attachments | `mog-core`, `mog-graph` |
| `mog-calendar` | Calendar module: today/week/list views (calendarView), create/update/delete events | `mog-core`, `mog-graph` |
| `mog-files` | Files module: search, download, upload (resumable), export/convert | `mog-core`, `mog-graph` |
| `mog-contacts` | Contacts module (post-MVP): list, search, read | `mog-core`, `mog-graph` |
| `mog-people` | People module (post-MVP): relevant, search | `mog-core`, `mog-graph` |
| `mog-tasks` | Tasks module (post-MVP): To Do lists/tasks CRUD | `mog-core`, `mog-graph` |
| `mog-directory` | Directory module (post-MVP): users/groups list, search, read, members | `mog-core`, `mog-graph` |

### Dependency Flow

```
mog-cli (binary)
├── mog-auth ──► mog-graph ──► mog-core
├── mog-mail ──► mog-graph ──► mog-core
├── mog-calendar ──► mog-graph ──► mog-core
├── mog-files ──► mog-graph ──► mog-core
├── mog-contacts ──► mog-graph ──► mog-core
├── mog-people ──► mog-graph ──► mog-core
├── mog-tasks ──► mog-graph ──► mog-core
└── mog-directory ──► mog-graph ──► mog-core
```

Zero cross-dependencies between workload modules.

### Key External Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` (derive) | CLI parsing, flag validation, shell completions |
| `reqwest` + `rustls` | HTTP client (no OpenSSL) |
| `tokio` | Async runtime |
| `serde` + `serde_json` | Serialization |
| `keyring` | OS credential storage (DPAPI, Keychain, libsecret) |
| `comfy-table` | Terminal table rendering |
| `tracing` | Structured logging |
| `zeroize` | Secure memory wiping |
| `chrono` | Date/time |
| `uuid` | Correlation IDs |
| `fs2` | File locking for token cache |

## Implementation Status

### MVP Modules (Implemented)

| Module | Commands | Status |
|--------|----------|--------|
| `auth` | login, logout, status, profile CRUD, explain-permissions, admin-consent-url | Implemented |
| `mail` | list, search, read, send, attachments list/download | Implemented |
| `calendar` | today, week, list, create, update, delete | Implemented |
| `files` | search, download, export, upload (resumable) | Implemented |
| `graph` | call (raw Graph escape hatch) | Implemented |
| `config` | show, set | Implemented |

### Post-MVP Modules (Scaffolded)

| Module | Commands | Status |
|--------|----------|--------|
| `contacts` | list, search, read | Scaffolded |
| `people` | relevant, search | Scaffolded |
| `tasks` | lists, list, create, update, complete, delete | Scaffolded |
| `directory` | users (list/search/read), groups (list/search/read/members) | Scaffolded |

### Security Hardening (Pending)

See `codereview.md` for full findings. Critical items:

1. **Token cache encryption** — Currently plain JSON. Needs keyring integration or AES-GCM encryption.
2. **Client secret zeroization** — Secrets read from stdin not wrapped in `Zeroizing`.
3. **URL parameter encoding** — File search queries need percent-encoding.
4. **Async file I/O** — Replace `std::fs` with `tokio::fs` in async contexts.

### Deferred Features (from roadmap.md)

| Feature | Target |
|---------|--------|
| JMESPath `--query` flag | v2 |
| Calendar recurrence creation | v2 |
| Runtime plugins (WASM) | v2+ |
| Sovereign cloud full testing | v2 |
| Scoop/apt/rpm packaging | v2 |
| Telemetry (opt-in) | v3+ |

## Development Workflow

### Building

```bash
cargo build --workspace          # Debug build
cargo build --workspace --release # Release build
cargo test --workspace           # All tests
```

### CI

GitHub Actions (`.github/workflows/ci.yml`) runs on push/PR to `main`:
- Build
- Test
- Clippy (warnings = errors, `RUSTFLAGS=-Dwarnings`)
- Format check (`cargo fmt --check`)
- Documentation generation

### Workspace Lints

```toml
[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
too_many_arguments = "allow"
```

### Specifications

The project is spec-driven. All feature implementations must align with:

- `spec.md` — Master application specification
- `openspec/specs/*/spec.md` — Per-module normative specs
- `roadmap.md` — Deferred items (do not implement unless explicitly requested)
- `openspec/changes/*/` — Change proposals with implementation checklists

## Configuration Paths

| Platform | Config Dir | Token Cache |
|----------|-----------|-------------|
| Windows | `%LOCALAPPDATA%\mog\` | `%LOCALAPPDATA%\mog\token_cache.json` |
| macOS | `~/.config/mog/` | Keychain (`mog-token-cache`) |
| Linux | `~/.config/mog/` | `~/.config/mog/token_cache.json` |

Override with `MOG_CONFIG_DIR` environment variable.

## Supported Cloud Environments

| Cloud | Authority | Graph Endpoint |
|-------|-----------|----------------|
| public | login.microsoftonline.com | graph.microsoft.com |
| gcc | login.microsoftonline.com | graph.microsoft.com |
| gcc-high | login.microsoftonline.us | graph.microsoft.us |
| dod | login.microsoftonline.us | dod-graph.microsoft.us |
| china | login.chinacloudapi.cn | microsoftgraph.chinacloudapi.cn |

Note: Sovereign clouds (gcc-high, dod, china) are untested in v1.
