# Tasks

## 1. Workspace and crate boundaries

- [x] Create the Rust workspace root and baseline crate layout.
- [x] Add foundational crates for CLI entry, shared types, and Graph transport.
- [x] Define crate dependency direction so workload modules do not depend on each other.

## 2. CLI contracts

- [x] Implement global flags and shared command parsing behavior.
- [x] Implement stdout/stderr discipline and output mode selection.
- [x] Implement exit code mapping and signal handling behavior.

## 3. Configuration and runtime behavior

- [x] Implement config discovery for Windows, macOS, and Linux.
- [x] Implement global config parsing and validation.
- [x] Implement non-interactive mode detection and failures.

## 4. Graph transport baseline

- [x] Implement request correlation headers and user agent formatting.
- [x] Implement retry, backoff, throttling, and pagination handling.
- [x] Implement deprecation and beta endpoint warnings to stderr.

## 5. Verification

- [x] Add unit tests for command parsing, config resolution, and exit code mapping.
- [x] Add contract tests for pagination and retry behavior.
- [x] Add completion generation and packaging validation to CI.
