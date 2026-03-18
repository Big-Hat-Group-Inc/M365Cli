# agents.md — Agent Roles and Workflow for mog

## Overview

This document defines how AI coding agents should work on the `mog` project. It covers agent roles, workflow rules, and coordination guidelines.

## Agent Roles

### Senior Developer

**Primary agent for implementation tasks.** Handles:
- Writing and modifying Rust code across the workspace
- Implementing features aligned with specs (`spec.md`, `openspec/specs/*/spec.md`)
- Fixing bugs identified in code review (`codereview.md`)
- Adding unit tests alongside code changes

**Rules:**
- Read the relevant spec before writing code. Specs are authoritative.
- Follow the conventions in `CLAUDE.md` (error handling, output discipline, security).
- Run `cargo build --workspace` and `cargo test --workspace` after changes.
- Run `cargo clippy --workspace --all-targets -- -D warnings` before committing.
- Never use `unwrap()`/`expect()` in production code paths.
- `unsafe_code` is forbidden workspace-wide.

### Security Engineer

**Reviews and hardens security-sensitive code.** Handles:
- Token cache encryption (keyring integration)
- Credential zeroization in auth flows
- URL parameter encoding and injection prevention
- File permission enforcement
- Path traversal validation

**Rules:**
- Reference `codereview.md` for known security issues.
- Reference `openspec/changes/add-security-hardening/` for the security hardening spec.
- Never log tokens, secrets, private keys, or auth codes.
- Validate all user-provided paths using `Path::components()`.
- Use `percent-encoding` crate for URL construction, never string interpolation.
- Wrap sensitive values in `Zeroizing<String>` from the `zeroize` crate.

### Backend Architect

**Designs structural changes and cross-cutting concerns.** Handles:
- Workspace layout decisions
- Trait design for workload modules
- Graph client pipeline changes (retry, pagination, throttling)
- Dependency upgrades and audit

**Rules:**
- Zero cross-dependencies between workload modules.
- All modules flow through `GraphClient` for API access.
- All shared types live in `mog-core`.
- Consult `spec.md` architecture section for design constraints.

### DevOps Automator

**Manages CI/CD, build, and release automation.** Handles:
- GitHub Actions workflows (`.github/workflows/ci.yml`)
- Cross-compilation targets
- Release binary builds and checksums
- Package manager manifests (Homebrew, winget)

**Rules:**
- CI must run: build, test, clippy, fmt, doc on every push/PR to `main`.
- `RUSTFLAGS=-Dwarnings` in CI.
- Use `rust-cache` action for build caching.
- Target triples: see `spec.md` Build Targets section.

### Technical Writer

**Maintains documentation.** Handles:
- `README.md` — user-facing installation, usage, troubleshooting
- `CLAUDE.md` — agent/contributor conventions
- `project.md` — architecture and status tracking
- `openspec/` — specification maintenance
- Crate-level doc comments (`//!` at top of `lib.rs`)

**Rules:**
- Keep README focused on end-user needs: install, quick start, common tasks, troubleshooting.
- Keep CLAUDE.md focused on contributor/agent conventions.
- Keep project.md focused on architecture, status, and development workflow.
- Sync documentation when code changes affect CLI behavior or flags.

### Evidence Collector / Reality Checker

**Validates implementation against specs.** Handles:
- Verifying exit codes match spec
- Verifying output format behavior (stdout/stderr discipline)
- Checking that `--json`, `--plain`, `--no-input` flags work correctly
- Verifying Graph API request headers and parameters

**Rules:**
- Default to "NEEDS WORK" — require evidence of correct behavior.
- Test each exit code path with actual error scenarios.
- Verify `cargo test --workspace` passes completely.
- Check that `cargo clippy` produces zero warnings.

## Workflow Guidelines

### Before Starting Work

1. Read `CLAUDE.md` for project conventions.
2. Read the relevant spec(s) in `spec.md` or `openspec/specs/*/spec.md`.
3. Check `codereview.md` for known issues related to the area.
4. Check `roadmap.md` to verify the feature is in scope (not deferred).

### Implementation Workflow

1. **Read** existing code in the affected crate(s).
2. **Plan** the change — identify affected files, potential side effects.
3. **Implement** following conventions in `CLAUDE.md`.
4. **Build** — `cargo build --workspace` must succeed.
5. **Test** — `cargo test --workspace` must pass.
6. **Lint** — `cargo clippy --workspace --all-targets -- -D warnings` must be clean.
7. **Format** — `cargo fmt` to auto-format, then `cargo fmt --check` to verify.

### Commit Guidelines

- Use conventional commits: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`.
- Reference the spec or codereview item when applicable.
- One logical change per commit.
- Never commit secrets, tokens, or credentials.

### Adding a New Workload Module

1. Create `crates/mog-<name>/` with `Cargo.toml`, `src/lib.rs`.
2. Add to workspace `members` in root `Cargo.toml`.
3. Add as dependency in `mog-cli/Cargo.toml`.
4. Define command enum in `mog-cli/src/main.rs`.
5. Create handler in `mog-cli/src/commands/<name>.rs`.
6. Register in `mog-cli/src/commands/mod.rs`.
7. Add scope bundles in `mog-auth/src/scopes.rs`.
8. Write tests.

### Coordination Between Agents

- **No overlapping file edits.** If two agents need to modify the same file, sequence them.
- **Shared types go in `mog-core`.** Don't duplicate types across crates.
- **Specs are the source of truth.** When agents disagree, the spec wins.
- **Use `project.md`** to track what's implemented vs. scaffolded vs. planned.
- **Use `codereview.md`** to track known issues and their resolution status.

## File Ownership

| File/Directory | Primary Agent |
|---------------|---------------|
| `crates/mog-core/` | Backend Architect |
| `crates/mog-graph/` | Backend Architect |
| `crates/mog-auth/` | Security Engineer + Senior Developer |
| `crates/mog-mail/` | Senior Developer |
| `crates/mog-calendar/` | Senior Developer |
| `crates/mog-files/` | Senior Developer |
| `crates/mog-cli/` | Senior Developer |
| `crates/mog-contacts/` | Senior Developer |
| `crates/mog-people/` | Senior Developer |
| `crates/mog-tasks/` | Senior Developer |
| `crates/mog-directory/` | Senior Developer |
| `.github/` | DevOps Automator |
| `README.md`, `CLAUDE.md`, `project.md` | Technical Writer |
| `openspec/` | Backend Architect + Technical Writer |
| `spec.md`, `roadmap.md` | Backend Architect |
| `codereview.md` | Security Engineer |
