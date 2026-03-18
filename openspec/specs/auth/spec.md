# Authentication and Profiles Specification

## ADDED Requirements

### Requirement: First-class profile management
The system SHALL manage named profiles containing tenant, client, cloud, auth strategy, and scope bundle metadata.

#### Scenario: default profile is selected automatically
- **GIVEN** multiple profiles exist and `defaultProfile` is configured
- **WHEN** a command runs without `--profile`
- **THEN** the CLI SHALL resolve the configured default profile.

#### Scenario: explicit profile overrides default
- **GIVEN** a default profile exists
- **WHEN** a command is run with `--profile automation`
- **THEN** the CLI SHALL use the explicitly requested profile.

### Requirement: Supported authentication strategies
The system SHALL support `device-code`, `browser`, `client-credentials`, `managed-identity`, and `federated` authentication strategies.

#### Scenario: delegated device-code login
- **GIVEN** a profile uses `device-code`
- **WHEN** `mog auth login` is invoked for that profile
- **THEN** the CLI SHALL start the device-code flow and persist the resulting token set on success.

#### Scenario: browser fallback for conditional access
- **GIVEN** device-code authentication is blocked by tenant policy
- **WHEN** the CLI detects the relevant Entra ID error condition
- **THEN** it SHALL recommend the browser strategy on stderr.

### Requirement: Secure token storage
The system SHALL store tokens in OS-native secure storage where available and use an encrypted local fallback only when native storage is unavailable.

#### Scenario: keyring available
- **GIVEN** the host platform provides a supported secure credential store
- **WHEN** tokens are cached
- **THEN** the CLI SHALL store them in that secure store.

#### Scenario: secure fallback
- **GIVEN** OS-native secure storage is unavailable
- **WHEN** tokens must be cached
- **THEN** the CLI SHALL use an encrypted local cache with restrictive file permissions.

### Requirement: Least-privilege scope bundles
The system SHALL map commands and requested services to predefined scope bundles rather than requiring users to construct arbitrary permission sets.

#### Scenario: readonly mail and calendar login
- **GIVEN** a user runs `mog auth login --services mail,calendar --readonly`
- **WHEN** scopes are resolved
- **THEN** the CLI SHALL request the least-privilege delegated scopes required for read-only mail and calendar operations.

### Requirement: Missing-scope remediation
The system SHALL detect when an active token lacks required scopes for a command and provide remediation guidance.

#### Scenario: command requires missing scope
- **GIVEN** the active token does not include a scope required by the selected command
- **WHEN** the command is invoked
- **THEN** the CLI SHALL fail with an authentication or authorization error
- **AND** it SHALL list the missing scopes
- **AND** it SHALL provide a remediation command using `mog auth login --add-scopes`.

### Requirement: Permission explanation helpers
The system SHALL explain permission requirements for commands and generate an admin consent URL for tenant administrators.

#### Scenario: explain permissions
- **GIVEN** a user runs `mog auth explain-permissions mail send`
- **WHEN** the command completes
- **THEN** it SHALL describe the Graph permissions required for that operation and why they are needed.

#### Scenario: admin consent helper
- **GIVEN** a profile contains tenant and client registration data
- **WHEN** a user runs `mog auth admin-consent-url`
- **THEN** the CLI SHALL output a valid admin consent URL for that app registration and tenant context.

### Requirement: Token lifecycle cleanup
The system SHALL remove cached credentials when a user logs out or deletes a profile.

#### Scenario: logout clears tokens
- **GIVEN** a profile has cached tokens
- **WHEN** `mog auth logout --profile <name>` succeeds
- **THEN** the cached tokens for that profile SHALL be deleted.

#### Scenario: deleting a profile removes auth state
- **GIVEN** a profile exists with associated cached credentials
- **WHEN** the profile is deleted
- **THEN** the profile metadata and cached auth state SHALL both be removed.
