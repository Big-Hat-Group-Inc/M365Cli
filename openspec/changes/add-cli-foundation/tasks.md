# Tasks

## 1. Workspace and crate boundaries

- [ ] Create the Rust workspace root and baseline crate layout.
- [ ] Add foundational crates for CLI entry, shared types, and Graph transport.
- [ ] Define crate dependency direction so workload modules do not depend on each other.

## 2. CLI contracts

- [ ] Implement global flags and shared command parsing behavior.
- [ ] Implement stdout/stderr discipline and output mode selection.
- [ ] Implement exit code mapping and signal handling behavior.

## 3. Configuration and runtime behavior

- [ ] Implement config discovery for Windows, macOS, and Linux.
- [ ] Implement global config parsing and validation.
- [ ] Implement non-interactive mode detection and failures.

## 4. Graph transport baseline

- [ ] Implement request correlation headers and user agent formatting.
- [ ] Implement retry, backoff, throttling, and pagination handling.
- [ ] Implement deprecation and beta endpoint warnings to stderr.

## 5. Verification

- [ ] Add unit tests for command parsing, config resolution, and exit code mapping.
- [ ] Add contract tests for pagination and retry behavior.
- [ ] Add completion generation and packaging validation to CI.
