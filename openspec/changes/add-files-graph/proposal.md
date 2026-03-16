# Change Proposal: Add Files and Raw Graph MVP

## Why

`spec.md` defines file operations and a raw Graph escape hatch as part of MVP. Files validate download, search, export, and resumable upload behavior, while the raw Graph command gives the CLI a controlled escape hatch for unsupported scenarios without blocking the MVP.

## What Changes

- Define the MVP files command set for list or search, download, upload, and export or conversion.
- Define resumable upload session behavior including chunk sizing and resume semantics.
- Define the raw Graph command surface for arbitrary HTTP methods, headers, and body sources.
- Define related output, validation, and security behavior for file content and raw API access.

## Scope

In scope:

- `mog files` MVP commands
- Resumable upload sessions
- Format conversion using Graph content export
- `mog graph call`

Out of scope:

- SharePoint administration
- Long-lived sync or watch features
- Post-MVP modules from the roadmap

## Impact

- Completes the MVP data-access surface described in `spec.md`.
- Provides a fallback path for Graph features not yet wrapped in first-class commands.
