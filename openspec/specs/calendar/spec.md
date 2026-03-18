# Calendar Specification

## ADDED Requirements

### Requirement: Day and week calendar views
The calendar module SHALL provide day and week range views using Graph `calendarView` so occurrences and exceptions are expanded within the requested range.

#### Scenario: today view
- **GIVEN** a user runs `mog calendar today`
- **WHEN** the command executes
- **THEN** it SHALL request the current day range via `calendarView`
- **AND** it SHALL return occurrences, exceptions, and single instances within that range.

#### Scenario: week view
- **GIVEN** a user runs `mog calendar week`
- **WHEN** the command executes
- **THEN** it SHALL request the current week range via `calendarView`.

### Requirement: Single event create
The calendar module SHALL support creating non-recurring events.

#### Scenario: create event
- **GIVEN** a user provides event details for a single occurrence
- **WHEN** `mog calendar create` succeeds
- **THEN** the CLI SHALL create the event via Graph and return the created event metadata.

### Requirement: Single event update and delete
The calendar module SHALL support updating and deleting individual events.

#### Scenario: update event
- **GIVEN** an existing single event identifier
- **WHEN** the user runs `mog calendar update <event-id> ...`
- **THEN** the CLI SHALL patch the event with the supplied changes.

#### Scenario: delete event
- **GIVEN** an existing event identifier
- **WHEN** the user runs `mog calendar delete <event-id>`
- **THEN** the CLI SHALL delete the event and return a successful command result.

### Requirement: Recurrence remains read-only in MVP
The calendar module SHALL support recurring events only through expanded read operations and SHALL NOT implement recurring event authoring in MVP.

#### Scenario: recurring event creation not supported
- **GIVEN** a user attempts to create a recurring event definition
- **WHEN** the request is parsed
- **THEN** the CLI SHALL reject the request with guidance that recurrence authoring is deferred from MVP.

### Requirement: Calendar permission separation
The calendar module SHALL require read permissions for view operations and read-write permissions for create, update, and delete operations.

#### Scenario: create attempted with read-only token
- **GIVEN** the active token includes read-only calendar permissions
- **WHEN** the user runs `mog calendar create`
- **THEN** the command SHALL fail with a permission error and remediation guidance.
