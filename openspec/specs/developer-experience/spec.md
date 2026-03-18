# Spec: Developer Experience & Project Quality

## Workspace Linting

**Given** a contributor runs `cargo clippy --workspace`
**Then** lint rules are inherited from `[workspace.lints]` in the root `Cargo.toml`
**And** no per-crate lint configuration is needed

**Given** the workspace lint configuration
**Then** `clippy::too_many_arguments` is allowed (existing pattern in `GraphClient::new`)
**And** `rust::unsafe_code` is forbidden

## Crate Documentation

**Given** a developer runs `cargo doc --workspace --no-deps`
**Then** every crate has a crate-level description visible in the generated docs
**And** no missing-docs warnings are emitted for the crate-level module

## Formatting

**Given** the workspace source code
**When** `cargo fmt --check` is run
**Then** no formatting differences are reported

## Continuous Integration

**Given** a push to any branch or a pull request
**Then** a GitHub Actions workflow runs the following checks:
  - `cargo build --workspace`
  - `cargo test --workspace`
  - `cargo clippy --workspace -- -D warnings`
  - `cargo fmt --check`
  - `cargo doc --workspace --no-deps` (with `-Dwarnings`)

**Given** any check fails
**Then** the workflow reports failure
