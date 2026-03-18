# Tasks: Developer Experience & Project Quality

## Phase 2: Workspace Linting

- [x] Add `[workspace.lints.clippy]` and `[workspace.lints.rust]` to root `Cargo.toml`
- [x] Add `[lints] workspace = true` to all 10 crate `Cargo.toml` files

## Phase 3: Crate-Level Documentation

- [x] Add `//!` doc comments to `mog-core/src/lib.rs`
- [x] Add `//!` doc comments to `mog-graph/src/lib.rs`
- [x] Add `//!` doc comments to `mog-auth/src/lib.rs`
- [x] Add `//!` doc comments to `mog-mail/src/lib.rs`
- [x] Add `//!` doc comments to `mog-calendar/src/lib.rs`
- [x] Add `//!` doc comments to `mog-files/src/lib.rs`
- [x] Add `//!` doc comments to `mog-contacts/src/lib.rs`
- [x] Add `//!` doc comments to `mog-people/src/lib.rs`
- [x] Add `//!` doc comments to `mog-tasks/src/lib.rs`
- [x] Add `//!` doc comments to `mog-directory/src/lib.rs`
- [x] Run `cargo fmt` across workspace

## Phase 4: CI/CD

- [x] Create `.github/workflows/ci.yml`

## Verification

- [x] `cargo build` compiles cleanly
- [x] `cargo test --workspace` passes (105 tests)
- [x] `cargo clippy --workspace -- -D warnings` clean
- [x] `cargo fmt --check` clean
- [x] `cargo doc --workspace --no-deps` builds without warnings
