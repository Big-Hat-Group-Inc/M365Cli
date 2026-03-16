# Tasks

## 1. Files module

- [ ] Implement files search against user drive content.
- [ ] Implement file download by item identifier.
- [ ] Implement file export or conversion using Graph content formatting.
- [ ] Implement upload for standard and resumable paths.

## 2. Upload session behavior

- [ ] Implement upload session creation and chunked transfer.
- [ ] Implement default chunk size of 5 MB with an override flag.
- [ ] Implement retry and resume behavior for interrupted uploads.

## 3. Raw Graph command

- [ ] Implement arbitrary method and path execution.
- [ ] Implement custom header input.
- [ ] Implement body input from file and stdin.
- [ ] Preserve stdout or stderr discipline for raw responses and errors.

## 4. Verification

- [ ] Add integration tests for search, download, export, and upload.
- [ ] Add contract tests for upload chunk boundaries and retry handling.
- [ ] Add tests for raw Graph request composition and response rendering.
