# Tasks (To Do) Specification

## ADDED Requirements

### Requirement: List task lists
The tasks module SHALL enumerate the user's To Do task lists.

#### Scenario: list task lists
- **GIVEN** a user runs `mog tasks lists`
- **WHEN** the command executes
- **THEN** it SHALL request `GET /me/todo/lists` and return all task lists.

### Requirement: List tasks in a list
The tasks module SHALL list tasks within a specified task list with optional status filtering.

#### Scenario: list tasks
- **GIVEN** a user runs `mog tasks list --list-id <id>`
- **WHEN** the command executes
- **THEN** it SHALL request `GET /me/todo/lists/{id}/tasks` with default `$select` fields (title, status, importance, dueDateTime).

#### Scenario: filter by status
- **GIVEN** a user runs `mog tasks list --list-id <id> --status completed`
- **WHEN** the command executes
- **THEN** it SHALL apply a `$filter` for `status eq 'completed'`.

### Requirement: Create a task
The tasks module SHALL support creating tasks with title, optional body, and optional due date.

#### Scenario: create task
- **GIVEN** a user provides task details
- **WHEN** `mog tasks create --list-id <id> --title "Buy milk"` succeeds
- **THEN** the CLI SHALL create the task via Graph and return the created task metadata.

### Requirement: Update a task
The tasks module SHALL support updating task fields via PATCH.

#### Scenario: update task
- **GIVEN** an existing task identifier
- **WHEN** the user runs `mog tasks update --list-id <id> --task-id <tid> --title "New title"`
- **THEN** the CLI SHALL patch the task with the supplied changes.

### Requirement: Complete a task
The tasks module SHALL provide a convenience command to mark a task as completed.

#### Scenario: complete task
- **GIVEN** an existing task identifier
- **WHEN** the user runs `mog tasks complete --list-id <id> --task-id <tid>`
- **THEN** the CLI SHALL set the task status to `completed` via PATCH.

### Requirement: Delete a task
The tasks module SHALL support deleting a task.

#### Scenario: delete task
- **GIVEN** an existing task identifier
- **WHEN** the user runs `mog tasks delete --list-id <id> --task-id <tid>`
- **THEN** the CLI SHALL delete the task and return a successful command result.

### Requirement: Tasks permission scope
The tasks module SHALL require `Tasks.ReadWrite` for all operations.

#### Scenario: missing tasks permission
- **GIVEN** the active token does not include `Tasks.ReadWrite`
- **WHEN** the user runs any tasks command
- **THEN** the command SHALL fail with a permission error and remediation guidance.
