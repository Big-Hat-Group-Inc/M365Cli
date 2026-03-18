# Change Proposal: Developer Experience & Project Quality

## Why

The code review identified several low-priority items that were explicitly deferred from the security hardening proposal: crate documentation, workspace linting, and CI/CD. These items improve contributor onboarding, code consistency, and merge confidence. Without them, the project lacks automated quality gates, has no crate-level documentation for `cargo doc`, and has inconsistent formatting.

## What Changes

- Add workspace-level clippy and rustdoc lint configuration via `[workspace.lints]`.
- Inherit lint configuration in all 10 crate `Cargo.toml` files.
- Add crate-level `//!` documentation to all `lib.rs` files and `main.rs`.
- Run `cargo fmt` to normalize all source formatting.
- Create a GitHub Actions CI workflow (`build`, `test`, `clippy`, `fmt --check`).

## Scope

In scope:

- Workspace lint configuration
- Crate-level doc comments (one paragraph per crate)
- `cargo fmt` pass across the entire workspace
- `.github/workflows/ci.yml` workflow

Out of scope:

- Function-level doc comments (too large a scope, incremental)
- Integration tests with mock Graph API (separate proposal)
- Feature flags for auth strategies (separate proposal)
- Keyring-based token storage (separate proposal)

## Impact

- **Contributor onboarding**: `cargo doc --workspace --open` produces useful documentation.
- **Code consistency**: Shared clippy rules prevent lint divergence across crates.
- **Merge confidence**: Every push and PR runs build + test + clippy + fmt.
- **Formatting**: One-time `cargo fmt` normalizes the entire codebase.
