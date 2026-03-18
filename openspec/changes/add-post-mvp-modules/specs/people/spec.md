# People Specification

## ADDED Requirements

### Requirement: Relevant people
The people module SHALL return the signed-in user's relevance-ranked people list.

#### Scenario: relevant people
- **GIVEN** a user runs `mog people relevant`
- **WHEN** the command executes
- **THEN** it SHALL request `GET /me/people` with default `$select` fields (displayName, scoredEmailAddresses, jobTitle, department)
- **AND** it SHALL support `--top` for result limiting.

### Requirement: Search people
The people module SHALL support searching people using the `$search` query parameter.

#### Scenario: search people
- **GIVEN** a user runs `mog people search --query "Jane"`
- **WHEN** the command executes
- **THEN** it SHALL request `GET /me/people?$search="Jane"` with default field selection
- **AND** it SHALL return matching people.

### Requirement: People permission scope
The people module SHALL require `People.Read` for all operations.

#### Scenario: missing people permission
- **GIVEN** the active token does not include `People.Read`
- **WHEN** the user runs any people command
- **THEN** the command SHALL fail with a permission error and remediation guidance.
