# CLAUDE.md — Project Instructions for Claude Code

## Project Overview

`mog` is a cross-platform Rust CLI for Microsoft 365 via Microsoft Graph. It follows gog-like ergonomics: single binary, multi-account profiles, JSON-first output, stdout/stderr discipline, and least-privilege auth.

## Repository Structure

```
mog/
├── Cargo.toml              # Workspace root — all deps managed here
├── crates/
│   ├── mog-cli/            # CLI entry point, clap command dispatch (binary: mog)
│   ├── mog-core/           # Shared types: MogError, ExitCode, OutputRenderer, ConfigStore
│   ├── mog-graph/          # Graph HTTP client: retry, pagination, throttling, upload sessions
│   ├── mog-auth/           # Auth flows, profiles, token cache, scope bundles
│   ├── mog-mail/           # Mail workload (list, search, read, send, attachments)
│   ├── mog-calendar/       # Calendar workload (today, week, list, create, update, delete)
│   ├── mog-files/          # Files workload (search, download, upload, export)
│   ├── mog-contacts/       # Contacts workload (post-MVP)
│   ├── mog-people/         # People workload (post-MVP)
│   ├── mog-tasks/          # To Do tasks workload (post-MVP)
│   └── mog-directory/      # Directory workload (post-MVP)
├── openspec/               # Normative specifications and change history
│   ├── specs/              # Current authoritative specs per module
│   └── changes/            # Original change proposals with tasks
├── .github/workflows/      # CI: build, test, clippy, fmt, doc
├── spec.md                 # Master application specification
├── roadmap.md              # Deferred items by priority tier
└── codereview.md           # Code review findings and action items
```

## Build and Test Commands

```bash
cargo build --workspace          # Build all crates
cargo test --workspace           # Run all tests
cargo clippy --workspace --all-targets -- -D warnings  # Lint
cargo fmt --check                # Format check
cargo doc --workspace --no-deps  # Generate docs
```

The binary name is `mog` (defined in `crates/mog-cli/Cargo.toml`).

## Key Conventions

### Output Discipline
- **Data goes to stdout. Hints, warnings, errors go to stderr. Always.**
- `--json` and `--plain` produce identical output regardless of TTY state.
- Non-interactive mode: auto-detected via TTY, or forced with `--no-input`.

### Exit Codes
| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Authentication failure |
| 3 | Authorization failure (403) |
| 4 | Not found (404) |
| 5 | Rate limited (429) |
| 6 | Network error |
| 7 | Conflict (409) |
| 8 | User cancelled |
| 130 | SIGINT |

### Error Handling
- Use `MogError` variants from `mog-core/src/error.rs`. Each variant maps to a specific exit code.
- Never use `unwrap()` or `expect()` in production paths. Use `?` with proper error conversion.
- Graph HTTP errors must map to the correct `MogError` variant (401 -> Auth, 403 -> Authz, 404 -> NotFound, 429 -> RateLimited).

### Security Rules
- **Never log tokens, secrets, or private keys.** Use `zeroize` for sensitive data in memory.
- Secrets must come via stdin (`--client-secret-stdin`), never command-line args.
- Token cache files and config files must have 0600 permissions on Unix.
- Use `$select` to minimize data exposure in Graph API calls.
- `unsafe_code` is `forbid` workspace-wide.

### Dependencies
- All shared dependencies are declared in workspace `[workspace.dependencies]` in root `Cargo.toml`.
- HTTP: `reqwest` with `rustls-tls` (no OpenSSL).
- Async: `tokio` with full features.
- Use `tokio::fs` for file I/O in async contexts, never `std::fs` in async functions.

### Adding a New Module
1. Create `crates/mog-<name>/` with `Cargo.toml` referencing `workspace = true` for version/edition.
2. Add the crate to workspace `members` in root `Cargo.toml`.
3. Add the crate as a dependency to `mog-cli/Cargo.toml`.
4. Create the subcommand enum in `mog-cli/src/main.rs`.
5. Create `mog-cli/src/commands/<name>.rs` with a `run()` function.
6. Add the module to `mog-cli/src/commands/mod.rs`.
7. Workload modules depend on `mog-graph` for API calls and `mog-core` for errors/output. Zero cross-dependencies between workload modules.

### Graph Client Usage
- Every Graph request must include `client-request-id` (UUID) and `User-Agent: mog/{version}`.
- Use `GraphClient` from `mog-graph` — never construct HTTP clients directly in workload modules.
- Default API version is v1.0. Beta requires explicit `--api-version beta`.
- Respect `Retry-After` headers. Max 4 concurrent requests per workload.

### Specifications
- `spec.md` is the master specification. `openspec/specs/*/spec.md` are per-module normative specs.
- When implementing features, cross-reference the relevant spec. Follow it precisely.
- `roadmap.md` tracks deferred items — do not implement v2/deferred features unless explicitly asked.

## CI Pipeline

GitHub Actions runs on push/PR to `main`: build, test, clippy (warnings = errors), fmt check, doc generation. See `.github/workflows/ci.yml`.

## Known Issues (from codereview.md)

- Token cache is plain JSON — needs keyring/encryption (critical)
- Client secrets not zeroized in auth flows (medium)
- File search queries not percent-encoded (medium)
- Blocking `std::fs` calls in async context in files module (medium)
- Some magic numbers need named constants
