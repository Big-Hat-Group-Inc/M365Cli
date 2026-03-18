# Directory Specification

## ADDED Requirements

### Requirement: List users
The directory module SHALL list users in the organization with default field selection (displayName, mail, jobTitle, department, officeLocation).

#### Scenario: list users
- **GIVEN** a user runs `mog directory users list`
- **WHEN** the command executes
- **THEN** it SHALL request `GET /users` with default `$select` fields
- **AND** it SHALL support `--top` for result limiting and `--filter` for OData filtering.

### Requirement: Search users
The directory module SHALL support searching users using `$search` with the `ConsistencyLevel: eventual` header.

#### Scenario: search users
- **GIVEN** a user runs `mog directory users search --query "John"`
- **WHEN** the command executes
- **THEN** it SHALL request `GET /users?$search="displayName:John"` with header `ConsistencyLevel: eventual`
- **AND** it SHALL include `$count=true` as required by the advanced query pattern.

### Requirement: Read a single user
The directory module SHALL support reading a single user by ID or UPN.

#### Scenario: read user
- **GIVEN** a user runs `mog directory users read <user-id>`
- **WHEN** the command executes
- **THEN** it SHALL request `GET /users/{id}` and return the user profile.

### Requirement: List groups
The directory module SHALL list groups in the organization with default field selection (displayName, mail, groupTypes, membershipRule).

#### Scenario: list groups
- **GIVEN** a user runs `mog directory groups list`
- **WHEN** the command executes
- **THEN** it SHALL request `GET /groups` with default `$select` fields.

### Requirement: Search groups
The directory module SHALL support searching groups using `$search` with the `ConsistencyLevel: eventual` header.

#### Scenario: search groups
- **GIVEN** a user runs `mog directory groups search --query "Engineering"`
- **WHEN** the command executes
- **THEN** it SHALL request `GET /groups?$search="displayName:Engineering"` with header `ConsistencyLevel: eventual`.

### Requirement: Read a single group
The directory module SHALL support reading a single group by ID.

#### Scenario: read group
- **GIVEN** a user runs `mog directory groups read <group-id>`
- **WHEN** the command executes
- **THEN** it SHALL request `GET /groups/{id}` and return the group record.

### Requirement: List group members
The directory module SHALL support listing members of a group.

#### Scenario: list group members
- **GIVEN** a user runs `mog directory groups members --group-id <id>`
- **WHEN** the command executes
- **THEN** it SHALL request `GET /groups/{id}/members` and return the member list.

### Requirement: Directory permission scopes
The directory module SHALL require `User.ReadBasic.All` for user operations and `GroupMember.Read.All` for group operations.

#### Scenario: missing directory permission
- **GIVEN** the active token does not include the required directory permissions
- **WHEN** the user runs any directory command
- **THEN** the command SHALL fail with a permission error and remediation guidance.
