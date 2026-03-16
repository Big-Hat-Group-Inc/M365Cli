# Raw Graph Specification

## ADDED Requirements

### Requirement: Raw Graph escape hatch
The system SHALL provide a `mog graph call` command that can issue authenticated Microsoft Graph requests using arbitrary supported HTTP methods and request paths.

#### Scenario: raw get request
- **GIVEN** a user runs `mog graph call GET /me/messages --top 5`
- **WHEN** the request is executed
- **THEN** the CLI SHALL send an authenticated GET request to the specified Graph path and render the response in the requested output format.

### Requirement: Custom request headers
The raw Graph command SHALL accept caller-specified HTTP headers.

#### Scenario: custom prefer header
- **GIVEN** a user provides `--header "Prefer: outlook.body-content-type=text"`
- **WHEN** the request is built
- **THEN** the CLI SHALL attach the provided header to the outgoing Graph request.

### Requirement: Request body input sources
The raw Graph command SHALL support request bodies from a file or stdin.

#### Scenario: body file input
- **GIVEN** a user runs `mog graph call POST /me/sendMail --body-file ./payload.json`
- **WHEN** the request is prepared
- **THEN** the CLI SHALL read the request body from the specified file.

#### Scenario: stdin input
- **GIVEN** JSON is piped to stdin and the user sets `--body-stdin`
- **WHEN** the request is prepared
- **THEN** the CLI SHALL read the request body from stdin.

### Requirement: Shared transport behavior applies to raw calls
The raw Graph command SHALL use the same authentication, versioning, retry, throttling, and correlation behavior as first-class modules.

#### Scenario: raw call is throttled
- **GIVEN** a raw Graph request receives a throttling response
- **WHEN** the transport layer handles the response
- **THEN** the raw command SHALL follow the shared retry and backoff policy.

### Requirement: Raw command preserves CLI output discipline
The raw Graph command SHALL preserve stdout for response data and stderr for diagnostics.

#### Scenario: raw call with warning output
- **GIVEN** the command receives a successful response and emits a beta or deprecation warning
- **WHEN** output is rendered
- **THEN** the response payload SHALL be written to stdout
- **AND** the warning SHALL be written to stderr.
