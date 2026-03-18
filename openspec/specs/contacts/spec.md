# Contacts Specification

## ADDED Requirements

### Requirement: List personal contacts
The contacts module SHALL list the signed-in user's personal contacts with default field selection (displayName, emailAddresses, mobilePhone, companyName).

#### Scenario: list contacts
- **GIVEN** a user runs `mog contacts list`
- **WHEN** the command executes
- **THEN** it SHALL request `GET /me/contacts` with default `$select` fields
- **AND** it SHALL support `--top` for result limiting and `--filter` for OData filtering.

### Requirement: Search contacts
The contacts module SHALL support searching contacts using the `$search` query parameter.

#### Scenario: search contacts
- **GIVEN** a user runs `mog contacts search --query "John"`
- **WHEN** the command executes
- **THEN** it SHALL request `GET /me/contacts?$search="John"` with default field selection
- **AND** it SHALL return matching contacts.

### Requirement: Read a single contact
The contacts module SHALL support reading a single contact by ID.

#### Scenario: read contact
- **GIVEN** a user runs `mog contacts read <contact-id>`
- **WHEN** the command executes
- **THEN** it SHALL request `GET /me/contacts/{id}` and return the full contact record.

### Requirement: Contacts permission scope
The contacts module SHALL require `Contacts.Read` for all operations.

#### Scenario: missing contacts permission
- **GIVEN** the active token does not include `Contacts.Read`
- **WHEN** the user runs any contacts command
- **THEN** the command SHALL fail with a permission error and remediation guidance.
