# Mail Specification

## ADDED Requirements

### Requirement: Mail list defaults to headers
The mail list command SHALL return message headers and summary metadata by default and SHALL NOT fetch or render full message bodies unless explicitly requested by a read operation.

#### Scenario: default mail list
- **GIVEN** a user runs `mog mail list`
- **WHEN** messages are fetched
- **THEN** the command SHALL request only the fields required for list output
- **AND** it SHALL exclude the full message body from the default result.

### Requirement: Mail search supports KQL
The mail search command SHALL support Graph mail search using KQL input and bounded result counts.

#### Scenario: kql search
- **GIVEN** a user runs `mog mail search --kql 'from:randiw AND hasAttachments:true' --top 25`
- **WHEN** the request is sent
- **THEN** the CLI SHALL perform a Graph-compatible search
- **AND** it SHALL limit the results to the requested maximum.

### Requirement: Mail read returns full message detail
The mail read command SHALL retrieve a full message representation for a selected message ID.

#### Scenario: read message
- **GIVEN** a valid message identifier
- **WHEN** the user runs `mog mail read <message-id>`
- **THEN** the CLI SHALL fetch the corresponding message detail and render the full message response in the requested output format.

### Requirement: Mail send supports attachments
The mail send command SHALL support sending mail with a subject, recipients, body content, and file attachments.

#### Scenario: send message with attachment
- **GIVEN** a user provides a recipient, subject, body file, and attachment path
- **WHEN** `mog mail send` succeeds
- **THEN** the CLI SHALL send the message via Graph
- **AND** it SHALL include the provided attachment in the sent message payload.

### Requirement: Attachment download
The mail module SHALL support downloading all attachments from a selected message into a target output directory.

#### Scenario: download attachments
- **GIVEN** a message contains one or more attachments
- **WHEN** the user runs `mog mail attachments download <message-id> --out-dir <path>`
- **THEN** the CLI SHALL download each attachment into the requested directory
- **AND** it SHALL report failures using stderr without corrupting stdout data output.

### Requirement: Mail permission separation
The mail module SHALL require read permissions for list, search, read, and attachment download operations, and send permissions for send operations.

#### Scenario: send attempted with read-only token
- **GIVEN** the active token lacks `Mail.Send`
- **WHEN** the user runs `mog mail send`
- **THEN** the command SHALL fail with a permission error and remediation guidance.
