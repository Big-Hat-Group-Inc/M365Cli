# Files Specification

## ADDED Requirements

### Requirement: Drive search
The files module SHALL support searching drive content using a user-provided query string.

#### Scenario: search drive content
- **GIVEN** a user runs `mog files search --q "quarterly results" --top 10`
- **WHEN** the request is sent
- **THEN** the CLI SHALL search the target drive hierarchy through Graph
- **AND** it SHALL limit results to the requested maximum.

### Requirement: File download
The files module SHALL support downloading a file by item identifier to a caller-specified output path.

#### Scenario: download file
- **GIVEN** a valid file item identifier and output path
- **WHEN** the user runs `mog files download --item-id <id> --out <path>`
- **THEN** the CLI SHALL download the file content to the requested path.

### Requirement: File export and conversion
The files module SHALL support Graph-backed content export for supported formats such as PDF.

#### Scenario: export to pdf
- **GIVEN** a source file type Graph can convert
- **WHEN** the user runs `mog files export --item-id <id> --format pdf --out <path>`
- **THEN** the CLI SHALL request converted content from Graph and write it to the requested path.

### Requirement: Resumable upload support
The files module SHALL support resumable uploads using Graph upload sessions with a default chunk size of 5 MB.

#### Scenario: resumable upload with default chunk size
- **GIVEN** a file is uploaded with `--resumable`
- **WHEN** no explicit chunk size is provided
- **THEN** the CLI SHALL create an upload session
- **AND** it SHALL upload the file in 5 MB chunks by default.

#### Scenario: resumable upload with custom chunk size
- **GIVEN** a user provides `--chunk-size <bytes>`
- **WHEN** the upload session runs
- **THEN** the CLI SHALL use the provided chunk size if it is valid for the upload workflow.

### Requirement: Interrupted upload handling
The files module SHALL surface resumable upload interruptions in a way that supports retry or continuation.

#### Scenario: upload interrupted by network error
- **GIVEN** an upload session is active
- **WHEN** a transient failure interrupts a chunk transfer
- **THEN** the CLI SHALL retry according to the shared retry policy
- **AND** it SHALL preserve session information needed to continue or report the failure.

### Requirement: Files permission separation
The files module SHALL require read permissions for search, download, and export, and read-write permissions for upload operations.

#### Scenario: upload attempted with read-only token
- **GIVEN** the active token lacks a write-capable files permission
- **WHEN** the user runs a file upload command
- **THEN** the command SHALL fail with a permission error and remediation guidance.
