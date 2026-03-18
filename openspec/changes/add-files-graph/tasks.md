# Tasks

## 1. Files module

- [x] Implement files search against user drive content.
- [x] Implement file download by item identifier.
- [x] Implement file export or conversion using Graph content formatting.
- [x] Implement upload for standard and resumable paths.

## 2. Upload session behavior

- [x] Implement upload session creation and chunked transfer.
- [x] Implement default chunk size of 5 MB with an override flag.
- [x] Implement retry and resume behavior for interrupted uploads.

## 3. Raw Graph command

- [x] Implement arbitrary method and path execution.
- [x] Implement custom header input.
- [x] Implement body input from file and stdin.
- [x] Preserve stdout or stderr discipline for raw responses and errors.

## 4. Verification

- [x] Add integration tests for search, download, export, and upload.
- [x] Add contract tests for upload chunk boundaries and retry handling.
- [x] Add tests for raw Graph request composition and response rendering.
