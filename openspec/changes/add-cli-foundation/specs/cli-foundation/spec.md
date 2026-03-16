# CLI Foundation Specification

## ADDED Requirements

### Requirement: Rust workspace foundation
The system SHALL be implemented as a Rust workspace with separate internal crates for CLI entry, shared core types, and Microsoft Graph transport.

#### Scenario: dependency direction remains one-way
- **GIVEN** the workspace contains `mog-cli`, `mog-core`, and `mog-graph`
- **WHEN** workload crates are added later
- **THEN** `mog-cli` may depend on the shared crates
- **AND** workload crates may depend on `mog-graph` and `mog-core`
- **AND** workload crates SHALL NOT depend on each other.

### Requirement: Stable global CLI behavior
The system SHALL expose shared global behavior for `--profile`, `--json`, `--plain`, `--no-input`, `--color`, and output selection across all commands.

#### Scenario: automation mode is stable
- **GIVEN** a command is run with `--json`
- **WHEN** stdout is attached to a TTY or redirected
- **THEN** the JSON payload written to stdout SHALL be stable and machine-readable in both cases.

#### Scenario: non-interactive mode fails fast
- **GIVEN** a command would require prompting
- **WHEN** stdin or stdout is not a TTY, or `--no-input` is set
- **THEN** the command SHALL fail without prompting
- **AND** it SHALL emit guidance on stderr.

### Requirement: Stdout and stderr discipline
The system SHALL write command data to stdout and operational hints, warnings, and errors to stderr.

#### Scenario: data output is preserved
- **GIVEN** a command succeeds and returns data
- **WHEN** it emits warnings or progress hints
- **THEN** the data payload SHALL remain on stdout
- **AND** warnings or progress output SHALL be written only to stderr.

### Requirement: Shared exit code contract
The system SHALL map common failure categories to documented exit codes.

#### Scenario: Graph authorization failure
- **GIVEN** a Graph request returns HTTP 403
- **WHEN** the CLI classifies the result
- **THEN** it SHALL exit with code `3`.

#### Scenario: network connectivity failure
- **GIVEN** Graph cannot be reached due to timeout or DNS failure
- **WHEN** retries are exhausted
- **THEN** the CLI SHALL exit with code `6`.

### Requirement: Graph request pipeline baseline
The system SHALL add a shared Graph transport pipeline that applies request IDs, user agent, retry handling, throttling compliance, and pagination normalization.

#### Scenario: request correlation headers are applied
- **GIVEN** any Graph request is issued
- **WHEN** the HTTP request is sent
- **THEN** it SHALL include a generated `client-request-id`
- **AND** it SHALL include `User-Agent: mog/{version} ({platform})`.

#### Scenario: retry-after is respected
- **GIVEN** Graph returns HTTP 429 with a `Retry-After` header
- **WHEN** the request is retried
- **THEN** the transport layer SHALL wait according to the server-provided delay before retrying.

#### Scenario: nextLink normalization
- **GIVEN** Graph returns `@odata.nextLink`
- **WHEN** the client follows the link
- **THEN** it SHALL handle both relative and absolute nextLink values correctly.

### Requirement: Graph version and deprecation warnings
The system SHALL default to Graph `v1.0` and warn on beta or deprecated endpoint usage.

#### Scenario: beta API warning
- **GIVEN** a command uses `--api-version beta`
- **WHEN** the request is prepared
- **THEN** the CLI SHALL emit a beta API warning on stderr.

#### Scenario: sunset warning
- **GIVEN** a Graph response contains a `Sunset` header
- **WHEN** the response is processed
- **THEN** the CLI SHALL emit a deprecation warning including the sunset date on stderr.

### Requirement: Platform configuration storage
The system SHALL support platform-specific config file discovery and optional override via `MOG_CONFIG_DIR`.

#### Scenario: config override is honored
- **GIVEN** the environment variable `MOG_CONFIG_DIR` is set
- **WHEN** the CLI reads or writes config
- **THEN** it SHALL use that directory instead of the default platform path.

### Requirement: Foundation-level verification
The system SHALL provide automated tests for command parsing, output contracts, config resolution, and shared Graph transport behavior.

#### Scenario: output snapshots protect automation
- **GIVEN** a plain or JSON command output contract
- **WHEN** unit tests run
- **THEN** the output format SHALL be verified with automated snapshot or equivalent regression tests.
